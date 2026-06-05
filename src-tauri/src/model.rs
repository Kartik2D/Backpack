use std::collections::HashMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct App {
    pub path: String,
    pub name: String,
    #[serde(default)]
    pub original_name: String,
    pub image: String,
    pub description: String,
    #[serde(default)]
    pub install_dir: Option<String>,
}

impl App {
    pub fn with_name(path: String, name: String, install_dir: Option<String>) -> Self {
        Self {
            path,
            name: name.clone(),
            original_name: name,
            image: String::new(),
            description: String::new(),
            install_dir,
        }
    }

    pub fn search_name(&self) -> &str {
        if self.original_name.is_empty() {
            &self.name
        } else {
            &self.original_name
        }
    }
}

#[derive(Default)]
pub struct AppList(pub Mutex<Vec<App>>);

#[derive(Clone)]
pub struct AppMetadata {
    pub name: Option<String>,
    pub image: Option<String>,
    pub description: Option<String>,
}

#[derive(Default)]
pub struct MetadataCache(pub Mutex<HashMap<String, AppMetadata>>);

#[derive(Clone, Serialize)]
pub struct IgdbSearchResult {
    pub id: i64,
    pub name: String,
    pub image: String,
    pub description: String,
}

pub struct Discovered {
    pub app: App,
}
