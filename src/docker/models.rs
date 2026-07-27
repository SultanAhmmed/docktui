use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Container {
    #[serde(rename = "ID")]
    pub id: String,
    pub image: String,
    pub names: String,
    pub state: String,
    pub status: String,
    pub ports: String,
}

impl Container {
    pub fn is_running(&self) -> bool {
        self.state == "running"
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Image {
    #[serde(rename = "ID")]
    pub id: String,
    pub repository: String,
    pub tag: String,
    pub size: String,
    #[serde(rename = "CreatedSince")]
    pub created_since: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComposeProject {
    pub name: String,
    pub status: String,
    #[serde(rename = "ConfigFiles")]
    pub config_files: String,
}
