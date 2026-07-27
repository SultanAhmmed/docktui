use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::docker::models::{ComposeProject, Container, Image};
use crate::docker::DockerClient;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    Containers,
    Logs,
    Images,
    Compose,
    Stats,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Filtering,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PendingAction {
    Start(String),
    Stop(String),
    Restart(String),
    Remove(String),
    ComposeUp(String),
    ComposeDown(String),
    PruneImages,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Favorites {
    pub container_ids: HashSet<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ContainerStats {
    pub cpu_percent: f64,
    pub memory_usage: u64,
    pub memory_limit: u64,
    pub history: Vec<u64>,
}

pub struct App {
    pub containers: Vec<Container>,
    pub filtered_containers: Vec<Container>,
    pub images: Vec<Image>,
    pub compose_projects: Vec<ComposeProject>,
    pub stats: HashMap<String, ContainerStats>,
    pub stats_error: Option<String>,
    pub last_stats_update: Option<Instant>, // NEW: To prevent lag

    pub selected: usize,
    pub running: bool,
    pub view: View,
    pub input_mode: InputMode,

    pub log_lines: Vec<String>,
    pub log_receiver: Option<mpsc::Receiver<String>>,
    pub log_container_name: String,

    pub filter_text: String,
    pub pending_action: Option<PendingAction>,
    pub status_message: Option<String>,

    pub favorites: Favorites,
    pub favorites_path: PathBuf,
}

impl App {

    pub async fn new() -> Result<Self> {
        let favorites_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("docktui")
            .join("favorites.json");

        let favorites = if favorites_path.exists() {
            let content = tokio::fs::read_to_string(&favorites_path).await?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Favorites::default()
        };

        let mut app = Self {
            containers: Vec::new(),
            filtered_containers: Vec::new(),
            images: Vec::new(),
            compose_projects: Vec::new(),
            stats: HashMap::new(),
            stats_error: None,
            last_stats_update: None,
            selected: 0,
            running: true,
            view: View::Containers,
            input_mode: InputMode::Normal,
            log_lines: Vec::new(),
            log_receiver: None,
            log_container_name: String::new(),
            filter_text: String::new(),
            pending_action: None,
            status_message: None, // Will hold our startup warning if needed
            favorites,
            favorites_path,
        };

        // ✨ GRACEFUL STARTUP: If Docker isn't ready, don't crash!
        // Just save the error as a status message and let the user press 'r' later.
        if let Err(e) = app.refresh().await {
            app.status_message = Some(format!("⚠️ Docker not ready: {}. Press 'r' to refresh.", e));
        }

        Ok(app)
    }

    pub async fn refresh(&mut self) -> Result<()> {
        self.containers = DockerClient::list_containers().await?;
        self.images = DockerClient::list_images().await?;
        self.compose_projects = DockerClient::list_compose_projects().await?;
        self.apply_filter();
        Ok(())
    }

    pub async fn update_stats(&mut self) {
        if self.view == View::Stats {
            // FIX: Only fetch stats once per second to prevent UI lag
            let now = Instant::now();
            let should_update = match self.last_stats_update {
                Some(last) => now.duration_since(last).as_secs_f32() >= 1.0,
                None => true,
            };

            if should_update {
                match DockerClient::get_stats().await {
                    Ok(stats) => {
                        self.stats_error = None;
                        self.stats = stats;
                        for (_id, stat) in self.stats.iter_mut() {
                            stat.history.push((stat.cpu_percent * 10.0) as u64);
                            if stat.history.len() > 60 {
                                stat.history.remove(0);
                            }
                        }
                        self.last_stats_update = Some(now);
                    }
                    Err(e) => {
                        self.stats_error = Some(format!("Error fetching stats: {}", e));
                    }
                }
            }
        }
    }

    pub fn apply_filter(&mut self) {
        if self.filter_text.is_empty() {
            self.filtered_containers = self.containers.clone();
        } else {
            let filter_lower = self.filter_text.to_lowercase();
            self.filtered_containers = self
                .containers
                .iter()
                .filter(|c| {
                    c.names.to_lowercase().contains(&filter_lower)
                        || c.image.to_lowercase().contains(&filter_lower)
                })
                .cloned()
                .collect();
        }
        if self.selected >= self.filtered_containers.len() {
            self.selected = self.filtered_containers.len().saturating_sub(1);
        }
    }

    pub fn next(&mut self) {
        let len = match self.view {
            View::Containers => self.filtered_containers.len(),
            View::Images => self.images.len(),
            View::Compose => self.compose_projects.len(),
            _ => 0,
        };
        if len > 0 {
            self.selected = (self.selected + 1) % len;
        }
    }

    pub fn previous(&mut self) {
        let len = match self.view {
            View::Containers => self.filtered_containers.len(),
            View::Images => self.images.len(),
            View::Compose => self.compose_projects.len(),
            _ => 0,
        };
        if len > 0 {
            self.selected = if self.selected == 0 { len - 1 } else { self.selected - 1 };
        }
    }

    pub fn open_logs(&mut self) {
        if let Some(container) = self.filtered_containers.get(self.selected) {
            self.log_container_name = container.names.clone();
            self.log_lines.clear();
            self.log_receiver = Some(DockerClient::stream_logs(&container.id));
            self.view = View::Logs;
        }
    }

    pub fn close_logs(&mut self) {
        self.log_receiver = None;
        self.log_lines.clear();
        self.view = View::Containers;
    }

    pub fn drain_logs(&mut self) {
        if let Some(rx) = &mut self.log_receiver {
            while let Ok(line) = rx.try_recv() {
                self.log_lines.push(line);
            }
            const MAX_LOG_LINES: usize = 1000;
            if self.log_lines.len() > MAX_LOG_LINES {
                let excess = self.log_lines.len() - MAX_LOG_LINES;
                self.log_lines.drain(..excess);
            }
        }
    }

    pub fn switch_view(&mut self, view: View) {
        self.view = view;
        self.selected = 0;
    }

    pub fn toggle_filter(&mut self) {
        self.input_mode = match self.input_mode {
            InputMode::Normal => InputMode::Filtering,
            InputMode::Filtering => InputMode::Normal,
        };
    }

    pub fn input_char(&mut self, c: char) {
        if self.input_mode == InputMode::Filtering {
            self.filter_text.push(c);
            self.apply_filter();
        }
    }

    pub fn input_backspace(&mut self) {
        if self.input_mode == InputMode::Filtering {
            self.filter_text.pop();
            self.apply_filter();
        }
    }

    pub fn toggle_favorite(&mut self) {
        if let Some(container) = self.filtered_containers.get(self.selected) {
            if self.favorites.container_ids.contains(&container.id) {
                self.favorites.container_ids.remove(&container.id);
            } else {
                self.favorites.container_ids.insert(container.id.clone());
            }
            let _ = self.save_favorites();
        }
    }

    async fn save_favorites(&self) -> Result<()> {
        if let Some(parent) = self.favorites_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let content = serde_json::to_string(&self.favorites)?;
        tokio::fs::write(&self.favorites_path, content).await?;
        Ok(())
    }

    pub fn start_container(&mut self) {
        if let Some(container) = self.filtered_containers.get(self.selected) {
            self.pending_action = Some(PendingAction::Start(container.id.clone()));
        }
    }

    pub fn stop_container(&mut self) {
        if let Some(container) = self.filtered_containers.get(self.selected) {
            self.pending_action = Some(PendingAction::Stop(container.id.clone()));
        }
    }

    pub fn restart_container(&mut self) {
        if let Some(container) = self.filtered_containers.get(self.selected) {
            self.pending_action = Some(PendingAction::Restart(container.id.clone()));
        }
    }

    pub fn remove_container(&mut self) {
        if let Some(container) = self.filtered_containers.get(self.selected) {
            self.pending_action = Some(PendingAction::Remove(container.id.clone()));
        }
    }

    pub fn compose_up(&mut self) {
        if let Some(project) = self.compose_projects.get(self.selected) {
            self.pending_action = Some(PendingAction::ComposeUp(project.name.clone()));
        }
    }

    pub fn compose_down(&mut self) {
        if let Some(project) = self.compose_projects.get(self.selected) {
            self.pending_action = Some(PendingAction::ComposeDown(project.name.clone()));
        }
    }

    pub fn prune_images(&mut self) {
        self.pending_action = Some(PendingAction::PruneImages);
    }

    pub async fn confirm_action(&mut self) -> Result<()> {
        if let Some(action) = self.pending_action.take() {
            let result = match action {
                PendingAction::Start(id) => DockerClient::start_container(&id).await,
                PendingAction::Stop(id) => DockerClient::stop_container(&id).await,
                PendingAction::Restart(id) => DockerClient::restart_container(&id).await,
                PendingAction::Remove(id) => DockerClient::remove_container(&id).await,
                PendingAction::ComposeUp(name) => DockerClient::compose_up(&name).await,
                PendingAction::ComposeDown(name) => DockerClient::compose_down(&name).await,
                PendingAction::PruneImages => DockerClient::prune_images().await,
            };

            self.status_message = Some(match result {
                Ok(_) => "✓ Action completed".to_string(),
                Err(e) => format!("✗ Error: {}", e),
            });

            self.refresh().await?;
        }
        Ok(())
    }

    pub fn cancel_action(&mut self) {
        self.pending_action = None;
    }
}
