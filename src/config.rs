use std::collections::HashMap;
use std::hash::Hash;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Config {
    #[serde(default)]
    pub projects: Vec<Project>
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Project {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub apps: Vec<App>,
    #[serde(skip)]
    pub icon_data: Option<String>,     // base64 data URL, derived at load time
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct App {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: Option<String>,
    pub confirm: bool,
    #[serde(skip)]
    pub icon_data: Option<String>,     // base64 data URL, derived at load time
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let s = std::fs::read_to_string(path)?;
        let mut config: Config = serde_json::from_str(&s)?;

        for project in &mut config.projects {
            if let Some(icon) = &project.icon {
                if icon.contains('.') {  // likely a file path, not an emoji
                    project.icon_data = Self::load_icon_as_base64(icon);
                }
            }
        }
        Ok(config)
    }

    pub fn save(&self, path: &str) -> anyhow::Result<(), std::io::Error> {
        let s = serde_json::to_string_pretty(self)?;
        std::fs::write(path, s)
    }

    pub fn load_icon_as_base64(path: &str) -> Option<String> {
        let bytes = std::fs::read(path).ok()?;
        let ext   = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");
        let mime = match ext {
            "svg"  => "image/svg+xml",
            "jpg" | "jpeg" => "image/jpeg",
            "webp" => "image/webp",
            _      => "image/png",
        };

        Some(format!("data:{mime};base64,{}", BASE64_STANDARD.encode(&bytes)))
    }
}

impl Default for Config {
    fn default() -> Self {
        Config { projects: vec![] }
    }
}