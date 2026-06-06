use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::model::{App, AppMetadata, IgdbSearchResult};

// Fields requested from IGDB. parent_game / version_parent come back as raw ids
// because we do not expand them (no `.id` suffix), so plain i64 works.
const IGDB_FIELDS: &str =
    "name,total_rating_count,version_parent,parent_game,cover.image_id,screenshots.image_id,summary";
// game_type values we keep: 0 main game, 8 remake, 9 remaster, 10 expanded game,
// 11 port. Everything else (dlc, expansion, bundle, season, pack, update, ...) is
// excluded so DLC-heavy titles do not crowd out the base game.
const IGDB_GAME_TYPES: &str = "(0,8,9,10,11)";
// IGDB caps a multiquery request at 10 sub-queries.
const IGDB_MULTIQUERY_BATCH_SIZE: usize = 10;
const IGDB_BATCH_LIMIT: usize = 10;
const IGDB_SEARCH_LIMIT: usize = 10;
const IGDB_RETRY_DELAY: Duration = Duration::from_millis(750);
const IGDB_MAX_ATTEMPTS: usize = 3;

// Multi-word phrases stripped from a title before searching.
const NOISE_PHRASES: &[&str] = &["game preview", "early access", "game of the year"];
// Standalone words stripped anywhere in a title (rarely part of a real title).
const NOISE_WORDS: &[&str] = &["demo", "beta", "alpha", "trial", "preview", "pc"];
// Marketing edition tiers stripped only when trailing, so "Forza Horizon 5
// Premium Edition" collapses to its base game. Words that denote a distinct
// IGDB entry we keep (e.g. "remastered", "port") are intentionally absent.
const EDITION_WORDS: &[&str] = &[
    "edition",
    "deluxe",
    "ultimate",
    "premium",
    "complete",
    "definitive",
    "goty",
    "anniversary",
    "standard",
    "enhanced",
    "bundle",
];

type MetadataResult<T> = Result<T, String>;

// Synchronous progress reporter used to surface phase messages to the UI. It is
// only ever invoked on the calling (blocking) thread, so no Send/Sync is needed.
pub type ProgressFn<'a> = &'a dyn Fn(&str);

// Tallies surfaced to the UI after a batch run: `requests` is every HTTP attempt
// to IGDB (retries included); `items` is every user-visible field (name, cover
// URL, summary) present across all returned game objects, counted at parse time.
#[derive(Default, Clone, Copy, Serialize)]
pub struct IgdbStats {
    pub requests: usize,
    pub items: usize,
}

// Count the user-visible fields IGDB returned for one game.
fn count_fields(game: &IgdbGame) -> usize {
    game.name.as_deref().filter(|s| !s.is_empty()).is_some() as usize
        + game
            .cover
            .as_ref()
            .and_then(|cover| cover.image_id.as_deref())
            .filter(|s| !s.is_empty())
            .is_some() as usize
        + first_screenshot_url(game).is_some() as usize
        + game.summary.as_deref().filter(|s| !s.is_empty()).is_some() as usize
}

#[derive(Deserialize)]
struct IgdbToken {
    access_token: String,
}

#[derive(Clone, Deserialize)]
struct IgdbCover {
    image_id: Option<String>,
}

#[derive(Clone, Deserialize)]
struct IgdbScreenshot {
    image_id: Option<String>,
}

#[derive(Clone, Deserialize)]
struct IgdbGame {
    id: Option<i64>,
    name: Option<String>,
    parent_game: Option<i64>,
    version_parent: Option<i64>,
    total_rating_count: Option<f64>,
    summary: Option<String>,
    cover: Option<IgdbCover>,
    screenshots: Option<Vec<IgdbScreenshot>>,
}

struct IgdbClient {
    client_id: String,
    access_token: String,
    http: reqwest::blocking::Client,
    stats: Cell<IgdbStats>,
}

impl IgdbClient {
    fn from_env() -> MetadataResult<Self> {
        let client_id = std::env::var("IGDB_CLIENT_ID")
            .or_else(|_| std::env::var("TWITCH_CLIENT_ID"))
            .map_err(|_| "IGDB_CLIENT_ID or TWITCH_CLIENT_ID is not set.".to_string())?;
        let http = reqwest::blocking::Client::new();
        let access_token = std::env::var("IGDB_ACCESS_TOKEN")
            .or_else(|_| std::env::var("TWITCH_ACCESS_TOKEN"))
            .ok()
            .map(Ok)
            .unwrap_or_else(|| {
                let client_secret = std::env::var("IGDB_CLIENT_SECRET")
                    .or_else(|_| std::env::var("TWITCH_CLIENT_SECRET"))
                    .map_err(|_| {
                        "Set IGDB_ACCESS_TOKEN, or set IGDB_CLIENT_SECRET/TWITCH_CLIENT_SECRET so Backpack can request one.".to_string()
                    })?;
                let response = http
                    .post("https://id.twitch.tv/oauth2/token")
                    .query(&[
                        ("client_id", client_id.as_str()),
                        ("client_secret", client_secret.as_str()),
                        ("grant_type", "client_credentials"),
                    ])
                    .send()
                    .map_err(|err| format!("Failed to request IGDB token: {err}"))?;
                let status = response.status();
                if !status.is_success() {
                    let body = response.text().unwrap_or_default();
                    return Err(format!("IGDB token request failed with {status}: {body}"));
                }
                response
                    .json::<IgdbToken>()
                    .map(|token| token.access_token)
                    .map_err(|err| format!("Failed to parse IGDB token response: {err}"))
            })?;

        Ok(Self {
            client_id,
            access_token,
            http,
            stats: Cell::new(IgdbStats::default()),
        })
    }

    fn stats(&self) -> IgdbStats {
        self.stats.get()
    }

    fn bump_requests(&self) {
        let mut stats = self.stats.get();
        stats.requests += 1;
        self.stats.set(stats);
    }

    fn add_items(&self, games: &[IgdbGame]) {
        let count: usize = games.iter().map(count_fields).sum();
        if count > 0 {
            let mut stats = self.stats.get();
            stats.items += count;
            self.stats.set(stats);
        }
    }

    // Single-title lookup for the manual search modal: one /games search call.
    fn search_games(&self, name: &str) -> MetadataResult<Vec<IgdbSearchResult>> {
        let cleaned = clean_title(name);
        let term = if cleaned.is_empty() {
            name.trim()
        } else {
            cleaned.as_str()
        };
        if term.is_empty() {
            return Ok(Vec::new());
        }

        let query = format!(
            "search \"{}\"; fields {IGDB_FIELDS}; limit {IGDB_SEARCH_LIMIT};",
            escape_igdb_string(term)
        );
        let games = self.fetch_games(query)?;
        Ok(rank_search_results(name, games))
    }

    fn fetch_games(&self, query: String) -> MetadataResult<Vec<IgdbGame>> {
        let games = parse_games_value(self.post_igdb_value("games", query)?)?;
        self.add_items(&games);
        Ok(games)
    }

    fn fetch_games_by_ids(&self, ids: &[i64]) -> MetadataResult<HashMap<i64, IgdbGame>> {
        if ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut out = HashMap::new();
        for chunk in ids.chunks(500) {
            let id_list = chunk
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join(",");
            let query = format!(
                "fields {IGDB_FIELDS}; where id = ({id_list}); limit {};",
                chunk.len()
            );
            out.extend(
                self.fetch_games(query)?
                    .into_iter()
                    .filter_map(|game| game.id.map(|id| (id, game))),
            );
        }
        Ok(out)
    }

    // Batch lookup: one prefix/game_type/sort sub-query per game, batched 10 to a
    // multiquery request. Returns the best game per key plus whether it was an
    // exact-name match (used to decide canonical-parent collapsing).
    fn fetch_best_games(
        &self,
        targets: &[SearchTarget],
        progress: ProgressFn,
    ) -> MetadataResult<HashMap<String, (IgdbGame, bool)>> {
        let mut best_by_key: HashMap<String, (IgdbGame, bool)> = HashMap::new();
        let total = targets.len().div_ceil(IGDB_MULTIQUERY_BATCH_SIZE);

        for (batch, chunk) in targets.chunks(IGDB_MULTIQUERY_BATCH_SIZE).enumerate() {
            progress(&format!(
                "Fetching metadata from IGDB… ({}/{total})",
                batch + 1
            ));
            let mut body = String::new();
            for (index, target) in chunk.iter().enumerate() {
                let _ = writeln!(
                    body,
                    "query games \"q{index}\" {{ fields {IGDB_FIELDS}; where name ~ \"{}\"* & game_type = {IGDB_GAME_TYPES}; sort total_rating_count desc; limit {IGDB_BATCH_LIMIT}; }};",
                    escape_igdb_string(&target.term)
                );
            }

            for (alias, games) in parse_multiquery_value(self.post_igdb_value("multiquery", body)?)?
            {
                self.add_items(&games);
                let Some(index) = alias
                    .strip_prefix('q')
                    .and_then(|n| n.parse::<usize>().ok())
                else {
                    continue;
                };
                let Some(target) = chunk.get(index) else {
                    continue;
                };
                if let Some(best) = pick_best(target, games) {
                    best_by_key.insert(target.key.clone(), best);
                }
            }
        }

        Ok(best_by_key)
    }

    fn post_igdb_value(&self, endpoint: &str, body: String) -> MetadataResult<Value> {
        let url = format!("https://api.igdb.com/v4/{endpoint}");
        let mut last_error = None;
        for attempt in 0..IGDB_MAX_ATTEMPTS {
            if attempt > 0 {
                thread::sleep(IGDB_RETRY_DELAY);
            }

            self.bump_requests();
            let Ok(response) = self
                .http
                .post(&url)
                .header("Client-ID", &self.client_id)
                .header("Authorization", format!("Bearer {}", self.access_token))
                .body(body.clone())
                .send()
            else {
                last_error = Some(format!("IGDB {endpoint} request failed before a response."));
                continue;
            };

            if response.status().as_u16() == 429 {
                last_error = Some(format!("IGDB {endpoint} request was rate limited."));
                continue;
            }

            let status = response.status();
            let text = response
                .text()
                .map_err(|err| format!("Failed to read IGDB {endpoint} response: {err}"))?;
            if !status.is_success() {
                let message = format!("IGDB {endpoint} request failed with {status}: {text}");
                eprintln!("{message}");
                return Err(message);
            }

            return serde_json::from_str::<Value>(&text).map_err(|err| {
                let message = format!("Failed to parse IGDB {endpoint} JSON: {err}. Body: {text}");
                eprintln!("{message}");
                message
            });
        }

        Err(last_error.unwrap_or_else(|| format!("IGDB {endpoint} request failed.")))
    }
}

// A single game to look up: `key` maps results back to apps, `term` is the
// cleaned prefix used in the IGDB filter.
struct SearchTarget {
    key: String,
    term: String,
}

pub fn search_igdb(query: &str) -> MetadataResult<Vec<IgdbSearchResult>> {
    IgdbClient::from_env()?.search_games(query)
}

fn parse_games_value(value: Value) -> MetadataResult<Vec<IgdbGame>> {
    serde_json::from_value::<Vec<IgdbGame>>(value)
        .map_err(|err| format!("Failed to parse IGDB games response: {err}"))
}

// Multiquery responses are arrays of `{ "name": <alias>, "result": [...] }`.
fn parse_multiquery_value(value: Value) -> MetadataResult<Vec<(String, Vec<IgdbGame>)>> {
    let items = value
        .as_array()
        .ok_or_else(|| format!("Expected IGDB multiquery array, got: {value}"))?;
    let mut out = Vec::with_capacity(items.len());

    for item in items {
        let name = item
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("IGDB multiquery item missing name: {item}"))?
            .to_string();
        let result = item
            .get("result")
            .or_else(|| item.get("results"))
            .cloned()
            .unwrap_or_else(|| Value::Array(Vec::new()));
        let games = parse_games_value(result)
            .map_err(|err| format!("Failed to parse IGDB multiquery result {name}: {err}"))?;
        out.push((name, games));
    }

    Ok(out)
}

fn igdb_image_url(image_id: &str, size: &str) -> String {
    format!("https://images.igdb.com/igdb/image/upload/t_{size}/{image_id}.jpg")
}

// First non-empty screenshot (landscape key art), built at 1080p.
fn first_screenshot_url(game: &IgdbGame) -> Option<String> {
    game.screenshots
        .as_ref()?
        .iter()
        .find_map(|screenshot| screenshot.image_id.as_deref())
        .filter(|id| !id.is_empty())
        .map(|id| igdb_image_url(id, "1080p"))
}

fn game_to_result(game: IgdbGame) -> IgdbSearchResult {
    let key_art = first_screenshot_url(&game).unwrap_or_default();
    IgdbSearchResult {
        id: game.id.unwrap_or_default(),
        name: game.name.unwrap_or_default(),
        image: game
            .cover
            .and_then(|cover| cover.image_id)
            .map(|id| igdb_image_url(&id, "cover_small"))
            .unwrap_or_default(),
        key_art,
        description: game.summary.unwrap_or_default(),
    }
}

fn game_to_metadata(game: IgdbGame) -> AppMetadata {
    let key_art = first_screenshot_url(&game);
    AppMetadata {
        name: game.name.filter(|name| !name.is_empty()),
        image: game
            .cover
            .and_then(|cover| cover.image_id)
            .map(|id| igdb_image_url(&id, "cover_big")),
        key_art,
        description: game.summary.filter(|summary| !summary.is_empty()),
    }
}

pub fn apply_metadata_to_app(app: &mut App, metadata: AppMetadata) {
    if let Some(name) = metadata.name.filter(|s| !s.is_empty()) {
        app.name = name;
    }
    if let Some(image) = metadata.image.filter(|s| !s.is_empty()) {
        app.image = image.replace("t_cover_small", "t_cover_big");
    }
    if let Some(key_art) = metadata.key_art.filter(|s| !s.is_empty()) {
        app.key_art = key_art;
    }
    if let Some(description) = metadata.description.filter(|s| !s.is_empty()) {
        app.description = description;
    }
}

fn escape_igdb_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

// Remove `(...)` / `[...]` blocks, trademark glyphs and underscores, then
// collapse whitespace. Casing is preserved (IGDB matching is case-insensitive).
fn clean_title(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut depth = 0i32;
    for ch in name.chars() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = (depth - 1).max(0),
            _ if depth > 0 => {}
            '_' | '™' | '®' | '©' => out.push(' '),
            other => out.push(other),
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

// Normalized key: cleaned title with subtitle separators flattened, lowercased.
// Used both as the app<->result key and for exact-name comparisons.
fn normalize_name(name: &str) -> String {
    clean_title(name)
        .replace([':', '-', '–', '—', '|', '·'], " ")
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn token_core(token: &str) -> String {
    token
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
}

// Cut a title at the first spaced subtitle separator (": ", " - ", " | ", ...).
// Bare hyphens are left intact so titles like "Spider-Man" survive.
fn first_segment(text: &str) -> &str {
    let mut cut = text.len();
    for sep in [": ", " - ", " – ", " — ", " | ", " ~ ", ":"] {
        if let Some(idx) = text.find(sep) {
            cut = cut.min(idx);
        }
    }
    text[..cut].trim()
}

// The prefix term used in the IGDB filter: cleaned, noise-stripped, subtitle
// removed, lowercased. Returns None when nothing usable remains.
fn batch_term(name: &str) -> Option<String> {
    let mut text = clean_title(name).to_lowercase();
    for phrase in NOISE_PHRASES {
        text = text.replace(phrase, " ");
    }

    let base = first_segment(&text);
    let mut tokens = base
        .split_whitespace()
        .filter(|token| !NOISE_WORDS.contains(&token_core(token).as_str()))
        .collect::<Vec<_>>();

    // Drop trailing edition markers ("... Premium Edition" -> "...").
    while tokens
        .last()
        .map(|token| EDITION_WORDS.contains(&token_core(token).as_str()))
        .unwrap_or(false)
    {
        tokens.pop();
    }

    // If cleaning emptied the term, fall back to the un-stripped base.
    let term = if tokens.is_empty() {
        base.split_whitespace().collect::<Vec<_>>().join(" ")
    } else {
        tokens.join(" ")
    };
    let term = term.trim().to_string();
    (!term.is_empty()).then_some(term)
}

fn search_target(name: &str) -> Option<SearchTarget> {
    let key = normalize_name(name);
    let term = batch_term(name)?;
    (!key.is_empty()).then_some(SearchTarget { key, term })
}

fn canonical_parent_id(game: &IgdbGame) -> Option<i64> {
    game.version_parent.or(game.parent_game)
}

// Choose the best game from a (DLC-filtered, rating-sorted) result set: an exact
// normalized-name match against the term or original key wins; otherwise take the
// top row, which is already the most popular surviving entry. The bool reports
// whether the pick was an exact match (so the caller can keep the user's actual
// edition instead of collapsing it to a canonical parent).
fn pick_best(target: &SearchTarget, games: Vec<IgdbGame>) -> Option<(IgdbGame, bool)> {
    if games.is_empty() {
        return None;
    }

    let exact = games.iter().position(|game| {
        game.name
            .as_deref()
            .map(|name| {
                let normalized = normalize_name(name);
                normalized == target.term || normalized == target.key
            })
            .unwrap_or(false)
    });

    match exact {
        Some(index) => games.into_iter().nth(index).map(|game| (game, true)),
        None => games.into_iter().next().map(|game| (game, false)),
    }
}

// For the manual modal: rank by exact-name match then popularity, dedupe by id.
fn rank_search_results(name: &str, games: Vec<IgdbGame>) -> Vec<IgdbSearchResult> {
    let target = normalize_name(name);
    let mut games = games;
    games.sort_by(|left, right| {
        let left_exact = left
            .name
            .as_deref()
            .map(|n| normalize_name(n) == target)
            .unwrap_or(false);
        let right_exact = right
            .name
            .as_deref()
            .map(|n| normalize_name(n) == target)
            .unwrap_or(false);
        right_exact.cmp(&left_exact).then_with(|| {
            let left_rating = left.total_rating_count.unwrap_or_default();
            let right_rating = right.total_rating_count.unwrap_or_default();
            right_rating
                .partial_cmp(&left_rating)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });

    let mut seen = HashSet::new();
    games
        .into_iter()
        .filter(|game| game.id.map(|id| seen.insert(id)).unwrap_or(false))
        .map(game_to_result)
        .take(IGDB_SEARCH_LIMIT)
        .collect()
}

// Resolve every name to metadata in as few API calls as possible:
// ceil(N/10) multiquery requests + 1 batched parent lookup.
fn lookup_many(
    igdb: &IgdbClient,
    names: &[String],
    progress: ProgressFn,
) -> MetadataResult<HashMap<String, AppMetadata>> {
    let mut seen = HashSet::new();
    let targets = names
        .iter()
        .filter_map(|name| search_target(name))
        .filter(|target| seen.insert(target.key.clone()))
        .collect::<Vec<_>>();
    if targets.is_empty() {
        return Ok(HashMap::new());
    }

    let best_by_key = igdb.fetch_best_games(&targets, progress)?;
    if best_by_key.is_empty() {
        return Ok(HashMap::new());
    }

    // Only collapse non-exact matches to their canonical parent; an exact match is
    // the title the user actually has (e.g. "Crysis Remastered"), so keep it.
    let parent_ids = best_by_key
        .values()
        .filter(|(_, exact)| !exact)
        .filter_map(|(game, _)| canonical_parent_id(game))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if !parent_ids.is_empty() {
        progress("Resolving editions…");
    }
    let parents = igdb.fetch_games_by_ids(&parent_ids)?;

    Ok(best_by_key
        .into_iter()
        .map(|(key, (game, exact))| {
            let canonical = if exact {
                game
            } else {
                canonical_parent_id(&game)
                    .and_then(|id| parents.get(&id).cloned())
                    .unwrap_or(game)
            };
            (key, game_to_metadata(canonical))
        })
        .collect())
}

fn missing_metadata(app: &App) -> bool {
    app.image.is_empty() || app.description.is_empty()
}

fn apply_fetched(
    apps: &mut [App],
    keys: &[String],
    fetched: &HashMap<String, AppMetadata>,
    only_missing: bool,
) {
    for (app, key) in apps.iter_mut().zip(keys) {
        if only_missing && !missing_metadata(app) {
            continue;
        }
        if let Some(metadata) = fetched.get(key).cloned() {
            apply_metadata_to_app(app, metadata);
        }
    }
}

pub fn enrich_missing(
    mut apps: Vec<App>,
    progress: ProgressFn,
) -> MetadataResult<(Vec<App>, IgdbStats)> {
    if apps.is_empty() {
        return Ok((apps, IgdbStats::default()));
    }

    let mut seen = HashSet::new();
    let mut names = Vec::new();
    let keys = apps
        .iter()
        .map(|app| {
            let search_name = app.search_name();
            let key = normalize_name(search_name);
            if missing_metadata(app) && !key.is_empty() && seen.insert(key.clone()) {
                names.push(search_name.to_string());
            }
            key
        })
        .collect::<Vec<_>>();
    if names.is_empty() {
        return Ok((apps, IgdbStats::default()));
    }

    let igdb = IgdbClient::from_env()?;
    let fetched = lookup_many(&igdb, &names, progress)?;
    apply_fetched(&mut apps, &keys, &fetched, true);
    Ok((apps, igdb.stats()))
}

pub fn enrich_new(apps: Vec<App>, progress: ProgressFn) -> MetadataResult<(Vec<App>, IgdbStats)> {
    enrich_missing(apps, progress)
}

pub fn refresh_all(
    mut apps: Vec<App>,
    progress: ProgressFn,
) -> MetadataResult<(Vec<App>, IgdbStats)> {
    if apps.is_empty() {
        return Ok((apps, IgdbStats::default()));
    }

    let mut seen = HashSet::new();
    let mut names = Vec::new();
    let keys = apps
        .iter()
        .map(|app| {
            let search_name = app.search_name();
            let key = normalize_name(search_name);
            if !key.is_empty() && seen.insert(key.clone()) {
                names.push(search_name.to_string());
            }
            key
        })
        .collect::<Vec<_>>();
    if names.is_empty() {
        return Ok((apps, IgdbStats::default()));
    }

    let igdb = IgdbClient::from_env()?;
    let fetched = lookup_many(&igdb, &names, progress)?;
    apply_fetched(&mut apps, &keys, &fetched, false);
    Ok((apps, igdb.stats()))
}
