use std::collections::HashMap;

use anyhow::{Context, Result};
use bollard::container::{ListContainersOptions, StatsOptions};
use bollard::Docker;
use futures_util::StreamExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

use super::models::{ComposeProject, Container, Image};
use crate::app::ContainerStats;

pub struct DockerClient;

impl DockerClient {
    pub async fn list_containers() -> Result<Vec<Container>> {
        let output = Command::new("docker")
            .args(["ps", "-a", "--format", "{{json .}}"])
            .output()
            .await
            .context("Failed to run docker")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("docker ps failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let containers: Vec<Container> = stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                serde_json::from_str(line)
                    .with_context(|| format!("Failed to parse container JSON: {}", line))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(containers)
    }

    pub async fn list_images() -> Result<Vec<Image>> {
        let output = Command::new("docker")
            .args(["images", "--format", "{{json .}}"])
            .output()
            .await
            .context("Failed to run docker images")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("docker images failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let images: Vec<Image> = stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                serde_json::from_str(line)
                    .with_context(|| format!("Failed to parse image JSON: {}", line))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(images)
    }

    pub async fn list_compose_projects() -> Result<Vec<ComposeProject>> {
        let output = Command::new("docker")
            .args(["compose", "ls", "--format", "json"])
            .output()
            .await
            .context("Failed to run docker compose ls")?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        // docker compose ls --format json outputs a JSON array
        let projects: Vec<ComposeProject> = serde_json::from_str(&stdout)
            .with_context(|| format!("Failed to parse compose projects: {}", stdout))?;
        Ok(projects)
    }

    pub fn stream_logs(container_id: &str) -> mpsc::Receiver<String> {
        let (tx, rx) = mpsc::channel(100);
        let id = container_id.to_string();

        tokio::spawn(async move {
            let child = match Command::new("docker")
                .args(["logs", "-f", "--tail", "100", &id])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(format!("[error] failed to spawn: {}", e)).await;
                    return;
                }
            };

            let stderr = match child.stderr {
                Some(s) => s,
                None => return,
            };

            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if tx.send(line).await.is_err() {
                    break;
                }
            }
        });

        rx
    }

    pub async fn start_container(id: &str) -> Result<()> {
        let output = Command::new("docker").args(["start", id]).output().await?;
        if !output.status.success() {
            anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    pub async fn stop_container(id: &str) -> Result<()> {
        let output = Command::new("docker").args(["stop", id]).output().await?;
        if !output.status.success() {
            anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    pub async fn restart_container(id: &str) -> Result<()> {
        let output = Command::new("docker").args(["restart", id]).output().await?;
        if !output.status.success() {
            anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    pub async fn remove_container(id: &str) -> Result<()> {
        let output = Command::new("docker").args(["rm", "-f", id]).output().await?;
        if !output.status.success() {
            anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    pub async fn compose_up(project: &str) -> Result<()> {
        let output = Command::new("docker")
            .args(["compose", "-p", project, "up", "-d"])
            .output()
            .await?;
        if !output.status.success() {
            anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    pub async fn compose_down(project: &str) -> Result<()> {
        let output = Command::new("docker")
            .args(["compose", "-p", project, "down"])
            .output()
            .await?;
        if !output.status.success() {
            anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    pub async fn prune_images() -> Result<()> {
        let output = Command::new("docker")
            .args(["image", "prune", "-f"])
            .output()
            .await?;
        if !output.status.success() {
            anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(())
    }

    pub async fn get_stats() -> Result<HashMap<String, ContainerStats>> {
        let docker = Docker::connect_with_local_defaults()?;
        let mut stats_map = HashMap::new();

        let containers = docker
            .list_containers(None::<ListContainersOptions<String>>)
            .await?;

        for container in containers {
            if let Some(id) = container.id {
                if let Some(name) = container.names.and_then(|n| n.first().cloned()) {
                    let clean_name = name.trim_start_matches('/').to_string();
                    let mut stream = docker.stats(&id, None::<StatsOptions>);

                    if let Some(Ok(stats)) = stream.next().await {
                        let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64
                            - stats.precpu_stats.cpu_usage.total_usage as f64;
                        let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) as f64
                            - stats.precpu_stats.system_cpu_usage.unwrap_or(0) as f64;

                        let cpu_percent = if system_delta > 0.0 {
                            (cpu_delta / system_delta) * 100.0
                        } else {
                            0.0
                        };

                        stats_map.insert(
                            clean_name,
                            ContainerStats {
                                cpu_percent,
                                memory_usage: stats.memory_stats.usage.unwrap_or(0),
                                memory_limit: stats.memory_stats.limit.unwrap_or(0),
                                history: Vec::new(),
                            },
                        );
                    }
                }
            }
        }
        Ok(stats_map)
    }
}
