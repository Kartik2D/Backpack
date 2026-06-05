use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use crate::model::{App, AppMetadata, IgdbSearchResult, MetadataCache};

#[derive(Deserialize)]
struct IgdbToken {
    access_token: String,
}

#[derive(Deserialize)]
struct IgdbCover {
    image_id: Option<String>,
}

#[derive(Deserialize)]
struct IgdbGame {
    id: Option<i64>,
    name: Option<String>,
    summary: Option<String>,
    cover: Option<IgdbCover>,
}

struct IgdbClient {
    client_id: String,
    access_token: String,
    http: reqwest::blocking::Client,
}

impl IgdbClient {
    fn from_env() -> Option<Self> {
        let client_id = std::env::var("IGDB_CLIENT_ID")
            .or_else(|_| std::env::var("TWITCH_CLIENT_ID"))
            .ok()?;
        let http = reqwest::blocking::Client::new();
        let access_token = std::env::var("IGDB_ACCESS_TOKEN")
            .or_else(|_| std::env::var("TWITCH_ACCESS_TOKEN"))
            .ok()
            .or_else(|| {
                let client_secret = std::env::var("IGDB_CLIENT_SECRET")
                    .or_else(|_| std::env::var("TWITCH_CLIENT_SECRET"))
                    .ok()?;
                http.post("https://id.twitch.tv/oauth2/token")
                    .query(&[
                        ("client_id", client_id.as_str()),
                        ("client_secret", client_secret.as_str()),
                        ("grant_type", "client_credentials"),
                    ])
                    .send()
                    .ok()?
                    .error_for_status()
                    .ok()?
                    .json::<IgdbToken>()
                    .ok()
                    .map(|t| t.access_token)
            })?;

        Some(Self {
            client_id,
            access_token,
            http,
        })
    }

    fn lookup_game(&self, name: &str) -> Option<AppMetadata> {
        self.search_games(name).into_iter().next().map(|result| {
            let image = if result.image.is_empty() {
                None
            } else {
                Some(result.image.replace("t_cover_small", "t_cover_big"))
            };
            AppMetadata {
                name: Some(result.name),
                image,
                description: (!result.description.is_empty()).then_some(result.description),
            }
        })
    }

    fn search_games(&self, name: &str) -> Vec<IgdbSearchResult> {
        let search = igdb_search_name(name);
        if search.is_empty() {
            return Vec::new();
        }
        let query = format!(
            "search \"{}\"; fields name,summary,cover.image_id; limit 12;",
            escape_igdb_string(&search)
        );
        self.http
            .post("https://api.igdb.com/v4/games")
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .body(query)
            .send()
            .ok()
            .and_then(|r| r.error_for_status().ok())
            .and_then(|r| r.json::<Vec<IgdbGame>>().ok())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|game| {
                let id = game.id?;
                let name = game.name?;
                Some(IgdbSearchResult {
                    id,
                    name,
                    image: game
                        .cover
                        .and_then(|cover| cover.image_id)
                        .map(|id| igdb_cover_url(&id, "cover_small"))
                        .unwrap_or_default(),
                    description: game.summary.unwrap_or_default(),
                })
            })
            .collect()
    }
}

pub fn search_igdb(query: &str) -> Vec<IgdbSearchResult> {
    IgdbClient::from_env()
        .map(|igdb| igdb.search_games(query))
        .unwrap_or_default()
}

fn igdb_cover_url(image_id: &str, size: &str) -> String {
    format!("https://images.igdb.com/igdb/image/upload/t_{size}/{image_id}.jpg")
}

pub fn apply_metadata_to_app(app: &mut App, metadata: AppMetadata) {
    if let Some(name) = metadata.name.filter(|s| !s.is_empty()) {
        app.name = name;
    }
    if let Some(image) = metadata.image.filter(|s| !s.is_empty()) {
        app.image = image.replace("t_cover_small", "t_cover_big");
    }
    if let Some(description) = metadata.description.filter(|s| !s.is_empty()) {
        app.description = description;
    }
}

fn escape_igdb_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn igdb_search_name(name: &str) -> String {
    name.replace(['_', '-'], " ")
        .replace('™', "")
        .replace('®', "")
        .trim()
        .to_string()
}

fn normalize_name(name: &str) -> String {
    igdb_search_name(name).to_lowercase()
}

fn lookup_many(igdb: &IgdbClient, names: &[String]) -> HashMap<String, AppMetadata> {
    if names.is_empty() {
        return HashMap::new();
    }

    let worker_count = names.len().min(8);
    let chunk_size = names.len().div_ceil(worker_count);

    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for chunk in names.chunks(chunk_size) {
            handles.push(scope.spawn(move || {
                chunk
                    .iter()
                    .filter_map(|name| {
                        let key = normalize_name(name);
                        igdb.lookup_game(name).map(|metadata| (key, metadata))
                    })
                    .collect::<Vec<_>>()
            }));
        }

        handles
            .into_iter()
            .flat_map(|handle| handle.join().unwrap_or_default())
            .collect()
    })
}

pub fn enrich_new(mut apps: Vec<App>, cache: &MetadataCache) -> Vec<App> {
    if apps.is_empty() {
        return apps;
    }

    let mut pending = Vec::new();
    {
        let cached = cache.0.lock().unwrap();
        let mut seen = HashSet::new();
        for app in &mut apps {
            if !app.image.is_empty() {
                continue;
            }

            let key = normalize_name(&app.name);
            if key.is_empty() {
                continue;
            }

            if let Some(metadata) = cached.get(&key).cloned() {
                apply_metadata_to_app(app, metadata);
            } else if seen.insert(key) {
                pending.push(app.name.clone());
            }
        }
    }

    let Some(igdb) = IgdbClient::from_env() else {
        return apps;
    };
    let fetched = lookup_many(&igdb, &pending);
    if fetched.is_empty() {
        return apps;
    }

    {
        let mut cached = cache.0.lock().unwrap();
        cached.extend(fetched.clone());
    }

    for app in &mut apps {
        if !app.image.is_empty() {
            continue;
        }
        let key = normalize_name(&app.name);
        if let Some(metadata) = fetched.get(&key).cloned() {
            apply_metadata_to_app(app, metadata);
        }
    }

    apps
}

pub fn refresh_all(mut apps: Vec<App>, cache: &MetadataCache) -> Vec<App> {
    if apps.is_empty() {
        return apps;
    }

    let mut seen = HashSet::new();
    let mut names = Vec::new();
    let keys = apps
        .iter()
        .map(|app| {
            let key = normalize_name(&app.name);
            if !key.is_empty() && seen.insert(key.clone()) {
                names.push(app.name.clone());
            }
            key
        })
        .collect::<Vec<_>>();

    let Some(igdb) = IgdbClient::from_env() else {
        return apps;
    };
    let fetched = lookup_many(&igdb, &names);
    if fetched.is_empty() {
        return apps;
    }

    {
        let mut cached = cache.0.lock().unwrap();
        cached.extend(fetched.clone());
    }

    for (app, key) in apps.iter_mut().zip(keys) {
        if let Some(metadata) = fetched.get(&key).cloned() {
            apply_metadata_to_app(app, metadata);
        }
    }

    apps
}
