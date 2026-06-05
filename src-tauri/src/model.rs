use std::collections::HashMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct App {
    pub path: String,
    pub name: String,
    pub image: String,
    pub description: String,
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
