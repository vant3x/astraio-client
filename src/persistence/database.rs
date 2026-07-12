use crate::error::AppError;
use directories::ProjectDirs;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Environment {
    pub id: i32,
    pub name: String,
    pub variables: Vec<(String, String)>,
    #[serde(default)]
    pub secret_keys: Vec<String>,
    pub default_endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestHistoryEntry {
    pub id: i32,
    pub method: String,
    pub url: String,
    pub status: Option<u16>,
    pub duration_ms: Option<u64>,
    pub timestamp: String,
    pub request_data: Option<String>,
    pub response_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Collection {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CollectionFolder {
    pub id: i32,
    pub collection_id: i32,
    pub name: String,
    pub parent_folder_id: Option<i32>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollectionBodyType {
    #[default]
    None,
    Text,
    Json,
    Xml,
    Html,
    FormUrlencoded,
    Multipart,
    Binary,
    Graphql,
}

impl fmt::Display for CollectionBodyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Text => write!(f, "text"),
            Self::Json => write!(f, "json"),
            Self::Xml => write!(f, "xml"),
            Self::Html => write!(f, "html"),
            Self::FormUrlencoded => write!(f, "form_urlencoded"),
            Self::Multipart => write!(f, "multipart"),
            Self::Binary => write!(f, "binary"),
            Self::Graphql => write!(f, "graphql"),
        }
    }
}

impl FromStr for CollectionBodyType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" | "" => Ok(Self::None),
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            "xml" => Ok(Self::Xml),
            "html" => Ok(Self::Html),
            "form_urlencoded" | "form-urlencoded" | "form" => Ok(Self::FormUrlencoded),
            "multipart" | "form-data" | "form_data" => Ok(Self::Multipart),
            "binary" | "octet-stream" => Ok(Self::Binary),
            "graphql" => Ok(Self::Graphql),
            _ => Ok(Self::Text),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollectionAuthType {
    #[default]
    None,
    Basic,
    Bearer,
    ApiKey,
    Oauth2,
    Digest,
}

impl fmt::Display for CollectionAuthType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Basic => write!(f, "basic"),
            Self::Bearer => write!(f, "bearer"),
            Self::ApiKey => write!(f, "api_key"),
            Self::Oauth2 => write!(f, "oauth2"),
            Self::Digest => write!(f, "digest"),
        }
    }
}

impl FromStr for CollectionAuthType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" | "" => Ok(Self::None),
            "basic" => Ok(Self::Basic),
            "bearer" | "token" => Ok(Self::Bearer),
            "api_key" | "apikey" | "api-key" => Ok(Self::ApiKey),
            "oauth2" | "oauth" => Ok(Self::Oauth2),
            "digest" => Ok(Self::Digest),
            _ => Ok(Self::None),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CollectionRequest {
    pub id: i32,
    pub collection_id: i32,
    pub folder_id: Option<i32>,
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
    #[serde(default)]
    pub body_type: CollectionBodyType,
    #[serde(default)]
    pub auth_type: CollectionAuthType,
    pub auth_data: Option<String>,
    pub params: Vec<(String, String)>,
    pub config_json: Option<String>,
    #[serde(default)]
    pub scripts: Option<String>,
    pub sort_order: i32,
}

impl std::fmt::Display for Collection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl std::fmt::Display for CollectionFolder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

fn get_db_path() -> std::result::Result<PathBuf, AppError> {
    let proj_dirs = ProjectDirs::from("com", "astranova", "client")
        .ok_or_else(|| AppError::Database("Failed to determine project directories".to_string()))?;
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir)
        .map_err(|e| AppError::Io(format!("Failed to create data directory: {}", e)))?;
    Ok(data_dir.join("astranova.db"))
}

pub fn init_schema(conn: &Connection) -> std::result::Result<(), AppError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS environments (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            variables TEXT NOT NULL
        )",
        [],
    )?;
    conn.execute(
        "ALTER TABLE environments ADD COLUMN default_endpoint TEXT",
        [],
    )
    .ok();
    conn.execute(
        "ALTER TABLE environments ADD COLUMN secret_keys TEXT NOT NULL DEFAULT '[]'",
        [],
    )
    .ok();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS request_history (
            id INTEGER PRIMARY KEY,
            method TEXT NOT NULL,
            url TEXT NOT NULL,
            status INTEGER,
            duration_ms INTEGER,
            timestamp TEXT NOT NULL,
            request_data TEXT,
            response_data TEXT
        )",
        [],
    )?;
    conn.execute(
        "ALTER TABLE request_history ADD COLUMN request_data TEXT",
        [],
    )
    .ok();
    conn.execute(
        "ALTER TABLE request_history ADD COLUMN response_data TEXT",
        [],
    )
    .ok();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS collections (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT
        )",
        [],
    )?;
    conn.execute(
        "ALTER TABLE collections ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0",
        [],
    )
    .ok();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS collection_folders (
            id INTEGER PRIMARY KEY,
            collection_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            parent_folder_id INTEGER,
            FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE,
            FOREIGN KEY (parent_folder_id) REFERENCES collection_folders(id) ON DELETE CASCADE
        )",
        [],
    )?;
    conn.execute(
        "ALTER TABLE collection_folders ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0",
        [],
    )
    .ok();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS collection_requests (
            id INTEGER PRIMARY KEY,
            collection_id INTEGER NOT NULL,
            folder_id INTEGER,
            name TEXT NOT NULL,
            method TEXT NOT NULL,
            url TEXT NOT NULL,
            headers TEXT NOT NULL DEFAULT '[]',
            body TEXT,
            body_type TEXT NOT NULL DEFAULT 'text',
            auth_type TEXT NOT NULL DEFAULT 'none',
            auth_data TEXT,
            params TEXT NOT NULL DEFAULT '[]',
            config_json TEXT,
            scripts TEXT,
            sort_order INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE,
            FOREIGN KEY (folder_id) REFERENCES collection_folders(id) ON DELETE CASCADE
        )",
        [],
    )?;
    conn.execute(
        "ALTER TABLE collection_requests ADD COLUMN scripts TEXT",
        [],
    )
    .ok();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

pub fn init() -> std::result::Result<Connection, AppError> {
    let db_path = get_db_path()?;
    let conn = Connection::open(db_path)?;
    init_schema(&conn)?;
    Ok(conn)
}

pub fn create_environment(conn: &Connection, name: &str) -> Result<Environment> {
    let variables: Vec<(String, String)> = Vec::new();
    let secret_keys: Vec<String> = Vec::new();
    let variables_json = serde_json::to_value(&variables)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
    let secret_keys_json = serde_json::to_value(&secret_keys)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
    conn.execute(
        "INSERT INTO environments (name, variables, secret_keys) VALUES (?1, ?2, ?3)",
        [
            name,
            &variables_json.to_string(),
            &secret_keys_json.to_string(),
        ],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Environment {
        id: id as i32,
        name: name.to_string(),
        variables,
        secret_keys,
        default_endpoint: None,
    })
}

pub fn get_environments(conn: &Connection) -> Result<Vec<Environment>> {
    let mut stmt = conn
        .prepare("SELECT id, name, variables, default_endpoint, secret_keys FROM environments")?;
    let env_iter = stmt.query_map([], |row| {
        let variables_json: String = row.get(2)?;
        let variables: Vec<(String, String)> =
            serde_json::from_str(&variables_json).unwrap_or_default();
        let secret_keys_json: String = row.get(4)?;
        let secret_keys: Vec<String> = serde_json::from_str(&secret_keys_json).unwrap_or_default();
        Ok(Environment {
            id: row.get(0)?,
            name: row.get(1)?,
            variables,
            secret_keys,
            default_endpoint: row.get(3)?,
        })
    })?;

    let mut environments = Vec::new();
    for env in env_iter {
        environments.push(env?);
    }
    Ok(environments)
}

pub fn update_environment(conn: &Connection, env: &Environment) -> Result<()> {
    let variables_json = serde_json::to_value(&env.variables)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
    let secret_keys_json = serde_json::to_value(&env.secret_keys)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
    conn.execute(
        "UPDATE environments SET name = ?1, variables = ?2, default_endpoint = ?3, secret_keys = ?4 WHERE id = ?5",
        params![
            &env.name,
            &variables_json.to_string(),
            &env.default_endpoint,
            &secret_keys_json.to_string(),
            &env.id.to_string(),
        ],
    )?;
    Ok(())
}

pub fn delete_environment(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM environments WHERE id = ?1", [&id.to_string()])?;
    Ok(())
}

pub fn get_app_setting(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        [key],
        |row| row.get(0),
    )
    .ok()
}

pub fn set_app_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
        [key, value],
    )?;
    Ok(())
}

pub fn save_request_history(
    conn: &Connection,
    method: &str,
    url: &str,
    status: Option<u16>,
    duration_ms: Option<u64>,
    request_data: Option<&str>,
    response_data: Option<&str>,
) -> Result<()> {
    let timestamp = crate::utils::timestamp_seconds();
    conn.execute(
        "INSERT INTO request_history (method, url, status, duration_ms, timestamp, request_data, response_data) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![method, url, status.map(|s| s as i64), duration_ms.map(|d| d as i64), timestamp, request_data, response_data],
    )?;
    Ok(())
}

pub fn get_request_history(conn: &Connection, limit: usize) -> Result<Vec<RequestHistoryEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, method, url, status, duration_ms, timestamp, request_data, response_data FROM request_history ORDER BY id DESC LIMIT ?1",
    )?;
    let entries = stmt.query_map([limit as i64], |row| {
        Ok(RequestHistoryEntry {
            id: row.get(0)?,
            method: row.get(1)?,
            url: row.get(2)?,
            status: row.get::<_, Option<i64>>(3)?.map(|s| s as u16),
            duration_ms: row.get::<_, Option<i64>>(4)?.map(|d| d as u64),
            timestamp: row.get(5)?,
            request_data: row.get(6)?,
            response_data: row.get(7)?,
        })
    })?;

    let mut result = Vec::new();
    for entry in entries {
        result.push(entry?);
    }
    Ok(result)
}

pub fn delete_request_history(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM request_history", [])?;
    Ok(())
}

pub fn delete_request_history_by_id(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM request_history WHERE id = ?1", [id])?;
    Ok(())
}

pub const DEFAULT_HISTORY_LIMIT: usize = 500;

pub fn trim_request_history(conn: &Connection, max_entries: usize) -> Result<()> {
    let count: i64 =
        conn.query_row("SELECT COUNT(*) FROM request_history", [], |row| row.get(0))?;
    let excess = count - max_entries as i64;
    if excess > 0 {
        conn.execute(
            "DELETE FROM request_history WHERE id IN (SELECT id FROM request_history ORDER BY id ASC LIMIT ?1)",
            [excess],
        )?;
    }
    Ok(())
}

fn map_history_row(row: &rusqlite::Row) -> rusqlite::Result<RequestHistoryEntry> {
    Ok(RequestHistoryEntry {
        id: row.get(0)?,
        method: row.get(1)?,
        url: row.get(2)?,
        status: row.get::<_, Option<i64>>(3)?.map(|s| s as u16),
        duration_ms: row.get::<_, Option<i64>>(4)?.map(|d| d as u64),
        timestamp: row.get(5)?,
        request_data: row.get(6)?,
        response_data: row.get(7)?,
    })
}

pub fn search_request_history(
    conn: &Connection,
    query: &str,
    method_filter: &str,
    limit: usize,
) -> Result<Vec<RequestHistoryEntry>> {
    let has_query = !query.is_empty();
    let has_method = !method_filter.is_empty();

    let sql = match (has_query, has_method) {
        (true, true) => "SELECT id, method, url, status, duration_ms, timestamp, request_data, response_data FROM request_history WHERE (url LIKE ?1 OR method LIKE ?1 OR request_data LIKE ?1 OR response_data LIKE ?1) AND method LIKE ?2 ORDER BY id DESC LIMIT ?3",
        (true, false) => "SELECT id, method, url, status, duration_ms, timestamp, request_data, response_data FROM request_history WHERE (url LIKE ?1 OR method LIKE ?1 OR request_data LIKE ?1 OR response_data LIKE ?1) ORDER BY id DESC LIMIT ?2",
        (false, true) => "SELECT id, method, url, status, duration_ms, timestamp, request_data, response_data FROM request_history WHERE method LIKE ?1 ORDER BY id DESC LIMIT ?2",
        (false, false) => "SELECT id, method, url, status, duration_ms, timestamp, request_data, response_data FROM request_history ORDER BY id DESC LIMIT ?1",
    };

    let pattern = format!("%{}%", query);
    let method_pattern = format!("%{}%", method_filter);
    let limit_val = limit as i64;

    let params: Vec<Box<dyn rusqlite::types::ToSql>> = match (has_query, has_method) {
        (true, true) => vec![
            Box::new(pattern),
            Box::new(method_pattern),
            Box::new(limit_val),
        ],
        (true, false) => vec![Box::new(pattern), Box::new(limit_val)],
        (false, true) => vec![Box::new(method_pattern), Box::new(limit_val)],
        (false, false) => vec![Box::new(limit_val)],
    };

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(sql)?;
    let entries: Vec<RequestHistoryEntry> = stmt
        .query_and_then(param_refs.as_slice(), map_history_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(entries)
}

pub fn get_request_history_entry_by_id(
    conn: &Connection,
    id: i32,
) -> Result<Option<RequestHistoryEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, method, url, status, duration_ms, timestamp, request_data, response_data FROM request_history WHERE id = ?1",
    )?;
    let mut entries = stmt.query_map([id], |row| {
        Ok(RequestHistoryEntry {
            id: row.get(0)?,
            method: row.get(1)?,
            url: row.get(2)?,
            status: row.get::<_, Option<i64>>(3)?.map(|s| s as u16),
            duration_ms: row.get::<_, Option<i64>>(4)?.map(|d| d as u64),
            timestamp: row.get(5)?,
            request_data: row.get(6)?,
            response_data: row.get(7)?,
        })
    })?;
    match entries.next() {
        Some(entry) => Ok(Some(entry?)),
        None => Ok(None),
    }
}

pub fn create_collection(
    conn: &Connection,
    name: &str,
    description: Option<&str>,
) -> Result<Collection> {
    conn.execute(
        "INSERT INTO collections (name, description) VALUES (?1, ?2)",
        params![name, description],
    )?;
    let id = conn.last_insert_rowid();
    let max_order: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), 0) FROM collections",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    Ok(Collection {
        id: id as i32,
        name: name.to_string(),
        description: description.map(|s| s.to_string()),
        sort_order: max_order + 1,
    })
}

pub fn get_collections(conn: &Connection) -> Result<Vec<Collection>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, sort_order FROM collections ORDER BY sort_order, name",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Collection {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            sort_order: row.get(3)?,
        })
    })?;
    rows.collect()
}

pub fn update_collection(conn: &Connection, collection: &Collection) -> Result<()> {
    conn.execute(
        "UPDATE collections SET name = ?1, description = ?2 WHERE id = ?3",
        params![collection.name, collection.description, collection.id],
    )?;
    Ok(())
}

pub fn delete_collection(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM collections WHERE id = ?1", [id])?;
    Ok(())
}

pub fn create_folder(
    conn: &Connection,
    collection_id: i32,
    name: &str,
    parent_folder_id: Option<i32>,
) -> Result<CollectionFolder> {
    let max_order: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), 0) FROM collection_folders WHERE collection_id = ?1 AND parent_folder_id IS ?2",
            params![collection_id, parent_folder_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    conn.execute(
        "INSERT INTO collection_folders (collection_id, name, parent_folder_id, sort_order) VALUES (?1, ?2, ?3, ?4)",
        params![collection_id, name, parent_folder_id, max_order + 1],
    )?;
    let id = conn.last_insert_rowid();
    Ok(CollectionFolder {
        id: id as i32,
        collection_id,
        name: name.to_string(),
        parent_folder_id,
        sort_order: max_order + 1,
    })
}

pub fn get_folders(conn: &Connection, collection_id: i32) -> Result<Vec<CollectionFolder>> {
    let mut stmt = conn.prepare(
        "SELECT id, collection_id, name, parent_folder_id, sort_order FROM collection_folders WHERE collection_id = ?1 ORDER BY sort_order, name",
    )?;
    let rows = stmt.query_map([collection_id], |row| {
        Ok(CollectionFolder {
            id: row.get(0)?,
            collection_id: row.get(1)?,
            name: row.get(2)?,
            parent_folder_id: row.get(3)?,
            sort_order: row.get(4)?,
        })
    })?;
    rows.collect()
}

pub fn delete_folder(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM collection_folders WHERE id = ?1", [id])?;
    Ok(())
}

pub fn rename_folder(conn: &Connection, id: i32, new_name: &str) -> Result<()> {
    conn.execute(
        "UPDATE collection_folders SET name = ?1 WHERE id = ?2",
        params![new_name, id],
    )?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct SaveRequestParams {
    pub collection_id: i32,
    pub folder_id: Option<i32>,
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
    pub body_type: CollectionBodyType,
    pub auth_type: CollectionAuthType,
    pub auth_data: Option<String>,
    pub params: Vec<(String, String)>,
    pub config_json: Option<String>,
    pub scripts: Option<String>,
}

impl SaveRequestParams {
    #[allow(dead_code)]
    pub fn new(collection_id: i32, name: &str, method: &str, url: &str) -> Self {
        Self {
            collection_id,
            folder_id: None,
            name: name.to_string(),
            method: method.to_string(),
            url: url.to_string(),
            headers: Vec::new(),
            body: None,
            body_type: CollectionBodyType::default(),
            auth_type: CollectionAuthType::default(),
            auth_data: None,
            params: Vec::new(),
            config_json: None,
            scripts: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn imported(
        collection_id: i32,
        folder_id: Option<i32>,
        name: &str,
        method: &str,
        url: &str,
        headers: &[(String, String)],
        body: Option<&str>,
        params: &[(String, String)],
    ) -> Self {
        Self {
            collection_id,
            folder_id,
            name: name.to_string(),
            method: method.to_string(),
            url: url.to_string(),
            headers: headers.to_vec(),
            body: body.map(str::to_string),
            body_type: CollectionBodyType::Text,
            auth_type: CollectionAuthType::None,
            auth_data: None,
            params: params.to_vec(),
            config_json: None,
            scripts: None,
        }
    }
}

pub fn save_collection_request(
    conn: &Connection,
    params: &SaveRequestParams,
) -> Result<CollectionRequest> {
    let headers_json = serde_json::to_string(&params.headers)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
    let params_json = serde_json::to_string(&params.params)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
    let max_order: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), 0) FROM collection_requests WHERE collection_id = ?1",
            [params.collection_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let body_type_str = params.body_type.to_string();
    let auth_type_str = params.auth_type.to_string();

    conn.execute(
        "INSERT INTO collection_requests (collection_id, folder_id, name, method, url, headers, body, body_type, auth_type, auth_data, params, config_json, scripts, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            params.collection_id,
            params.folder_id,
            params.name,
            params.method,
            params.url,
            headers_json,
            params.body,
            body_type_str,
            auth_type_str,
            params.auth_data,
            params_json,
            params.config_json,
            params.scripts,
            max_order + 1,
        ],
    )?;
    let id = conn.last_insert_rowid();
    Ok(CollectionRequest {
        id: id as i32,
        collection_id: params.collection_id,
        folder_id: params.folder_id,
        name: params.name.clone(),
        method: params.method.clone(),
        url: params.url.clone(),
        headers: params.headers.clone(),
        body: params.body.clone(),
        body_type: params.body_type.clone(),
        auth_type: params.auth_type.clone(),
        auth_data: params.auth_data.clone(),
        params: params.params.clone(),
        config_json: params.config_json.clone(),
        scripts: params.scripts.clone(),
        sort_order: max_order + 1,
    })
}

pub fn get_collection_requests(
    conn: &Connection,
    collection_id: i32,
    folder_id: Option<i32>,
) -> Result<Vec<CollectionRequest>> {
    let mut stmt = conn.prepare(
        "SELECT id, collection_id, folder_id, name, method, url, headers, body, body_type, auth_type, auth_data, params, config_json, scripts, sort_order FROM collection_requests WHERE collection_id = ?1 AND folder_id IS ?2 ORDER BY sort_order",
    )?;
    let rows = stmt.query_map(params![collection_id, folder_id], |row| {
        parse_collection_request(row)
    })?;
    rows.collect()
}

fn parse_collection_request(row: &rusqlite::Row) -> rusqlite::Result<CollectionRequest> {
    let headers_json: String = row.get(6)?;
    let params_json: String = row.get(11)?;
    let body_type_str: String = row.get(8)?;
    let auth_type_str: String = row.get(9)?;
    Ok(CollectionRequest {
        id: row.get(0)?,
        collection_id: row.get(1)?,
        folder_id: row.get(2)?,
        name: row.get(3)?,
        method: row.get(4)?,
        url: row.get(5)?,
        headers: serde_json::from_str(&headers_json).unwrap_or_default(),
        body: row.get(7)?,
        body_type: body_type_str.parse().unwrap_or_default(),
        auth_type: auth_type_str.parse().unwrap_or_default(),
        auth_data: row.get(10)?,
        params: serde_json::from_str(&params_json).unwrap_or_default(),
        config_json: row.get(12)?,
        scripts: row.get(13)?,
        sort_order: row.get(14)?,
    })
}

pub fn rename_collection_request(conn: &Connection, id: i32, new_name: &str) -> Result<()> {
    conn.execute(
        "UPDATE collection_requests SET name = ?1 WHERE id = ?2",
        params![new_name, id],
    )?;
    Ok(())
}

#[allow(dead_code)]
pub fn move_collection_request(
    conn: &Connection,
    id: i32,
    new_folder_id: Option<i32>,
) -> Result<()> {
    conn.execute(
        "UPDATE collection_requests SET folder_id = ?1 WHERE id = ?2",
        params![new_folder_id, id],
    )?;
    Ok(())
}

pub fn delete_collection_request(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM collection_requests WHERE id = ?1", [id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS environments (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                variables TEXT NOT NULL
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "ALTER TABLE environments ADD COLUMN default_endpoint TEXT",
            [],
        )
        .ok();
        conn.execute(
            "ALTER TABLE environments ADD COLUMN secret_keys TEXT NOT NULL DEFAULT '[]'",
            [],
        )
        .ok();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS collections (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                sort_order INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS collection_folders (
                id INTEGER PRIMARY KEY,
                collection_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                parent_folder_id INTEGER,
                sort_order INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS collection_requests (
                id INTEGER PRIMARY KEY,
                collection_id INTEGER NOT NULL,
                folder_id INTEGER,
                name TEXT NOT NULL,
                method TEXT NOT NULL,
                url TEXT NOT NULL,
                headers TEXT NOT NULL DEFAULT '[]',
                body TEXT,
                body_type TEXT NOT NULL DEFAULT 'text',
                auth_type TEXT NOT NULL DEFAULT 'none',
                auth_data TEXT,
                params TEXT NOT NULL DEFAULT '[]',
                config_json TEXT,
                scripts TEXT,
                sort_order INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn create_and_get_environment() {
        let conn = setup_test_db();
        let env = create_environment(&conn, "test-env").unwrap();
        assert_eq!(env.name, "test-env");
        assert!(env.variables.is_empty());
        assert!(env.default_endpoint.is_none());

        let envs = get_environments(&conn).unwrap();
        assert_eq!(envs.len(), 1);
        assert_eq!(envs[0].name, "test-env");
    }

    #[test]
    fn create_multiple_environments() {
        let conn = setup_test_db();
        create_environment(&conn, "env-1").unwrap();
        create_environment(&conn, "env-2").unwrap();
        create_environment(&conn, "env-3").unwrap();

        let envs = get_environments(&conn).unwrap();
        assert_eq!(envs.len(), 3);
    }

    #[test]
    fn update_environment_name() {
        let conn = setup_test_db();
        let mut env = create_environment(&conn, "original").unwrap();
        env.name = "updated".to_string();
        update_environment(&conn, &env).unwrap();

        let envs = get_environments(&conn).unwrap();
        assert_eq!(envs[0].name, "updated");
    }

    #[test]
    fn update_environment_variables() {
        let conn = setup_test_db();
        let mut env = create_environment(&conn, "with-vars").unwrap();
        env.variables = vec![
            ("API_URL".to_string(), "https://api.example.com".to_string()),
            ("TOKEN".to_string(), "abc123".to_string()),
        ];
        update_environment(&conn, &env).unwrap();

        let envs = get_environments(&conn).unwrap();
        assert_eq!(envs[0].variables.len(), 2);
        assert_eq!(envs[0].variables[0].0, "API_URL");
        assert_eq!(envs[0].variables[0].1, "https://api.example.com");
    }

    #[test]
    fn update_environment_endpoint() {
        let conn = setup_test_db();
        let mut env = create_environment(&conn, "with-endpoint").unwrap();
        env.default_endpoint = Some("https://api.example.com/v1".to_string());
        update_environment(&conn, &env).unwrap();

        let envs = get_environments(&conn).unwrap();
        assert_eq!(
            envs[0].default_endpoint,
            Some("https://api.example.com/v1".to_string())
        );
    }

    #[test]
    fn delete_existing_environment() {
        let conn = setup_test_db();
        let env = create_environment(&conn, "to-delete").unwrap();
        delete_environment(&conn, env.id).unwrap();

        let envs = get_environments(&conn).unwrap();
        assert!(envs.is_empty());
    }

    #[test]
    fn delete_nonexistent_environment_does_not_fail() {
        let conn = setup_test_db();
        let result = delete_environment(&conn, 999);
        assert!(result.is_ok());
    }

    #[test]
    fn environment_display() {
        let env = Environment {
            id: 1,
            name: "my-env".to_string(),
            variables: vec![],
            secret_keys: vec![],
            default_endpoint: None,
        };
        assert_eq!(env.to_string(), "my-env");
    }

    #[test]
    fn environment_clone_and_eq() {
        let env = Environment {
            id: 1,
            name: "clone-test".to_string(),
            variables: vec![("k".to_string(), "v".to_string())],
            secret_keys: vec![],
            default_endpoint: None,
        };
        let cloned = env.clone();
        assert_eq!(env, cloned);
    }

    #[test]
    fn save_and_get_request_history() {
        let conn = setup_test_db();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS request_history (
                id INTEGER PRIMARY KEY,
                method TEXT NOT NULL,
                url TEXT NOT NULL,
                status INTEGER,
                duration_ms INTEGER,
                timestamp TEXT NOT NULL,
                request_data TEXT,
                response_data TEXT
            )",
            [],
        )
        .unwrap();

        save_request_history(
            &conn,
            "GET",
            "https://example.com",
            Some(200),
            Some(150),
            None,
            None,
        )
        .unwrap();
        save_request_history(
            &conn,
            "POST",
            "https://api.test.com",
            Some(201),
            Some(300),
            None,
            None,
        )
        .unwrap();

        let history = get_request_history(&conn, 10).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].method, "POST");
        assert_eq!(history[1].method, "GET");
    }

    #[test]
    fn delete_request_history_clears_all() {
        let conn = setup_test_db();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS request_history (
                id INTEGER PRIMARY KEY,
                method TEXT NOT NULL,
                url TEXT NOT NULL,
                status INTEGER,
                duration_ms INTEGER,
                timestamp TEXT NOT NULL,
                request_data TEXT,
                response_data TEXT
            )",
            [],
        )
        .unwrap();

        save_request_history(
            &conn,
            "GET",
            "https://example.com",
            Some(200),
            Some(100),
            None,
            None,
        )
        .unwrap();
        delete_request_history(&conn).unwrap();
        let history = get_request_history(&conn, 10).unwrap();
        assert!(history.is_empty());
    }

    #[test]
    fn request_history_limit() {
        let conn = setup_test_db();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS request_history (
                id INTEGER PRIMARY KEY,
                method TEXT NOT NULL,
                url TEXT NOT NULL,
                status INTEGER,
                duration_ms INTEGER,
                timestamp TEXT NOT NULL,
                request_data TEXT,
                response_data TEXT
            )",
            [],
        )
        .unwrap();

        for i in 0..5 {
            save_request_history(
                &conn,
                "GET",
                &format!("https://example.com/{}", i),
                Some(200),
                Some(100),
                None,
                None,
            )
            .unwrap();
        }

        let history = get_request_history(&conn, 3).unwrap();
        assert_eq!(history.len(), 3);
    }

    #[test]
    fn create_and_get_collection() {
        let conn = setup_test_db();
        let col = create_collection(&conn, "My API", Some("All endpoints")).unwrap();
        assert_eq!(col.name, "My API");
        assert_eq!(col.description, Some("All endpoints".to_string()));

        let cols = get_collections(&conn).unwrap();
        assert_eq!(cols.len(), 1);
        assert_eq!(cols[0].name, "My API");
    }

    #[test]
    fn create_multiple_collections() {
        let conn = setup_test_db();
        create_collection(&conn, "API v1", None).unwrap();
        create_collection(&conn, "API v2", None).unwrap();
        create_collection(&conn, "Auth", None).unwrap();

        let cols = get_collections(&conn).unwrap();
        assert_eq!(cols.len(), 3);
        let names: Vec<&str> = cols.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"API v1"));
        assert!(names.contains(&"API v2"));
        assert!(names.contains(&"Auth"));
    }

    #[test]
    fn update_collection_test() {
        let conn = setup_test_db();
        let mut col = create_collection(&conn, "Old Name", None).unwrap();
        col.name = "New Name".to_string();
        col.description = Some("Updated desc".to_string());
        update_collection(&conn, &col).unwrap();

        let cols = get_collections(&conn).unwrap();
        assert_eq!(cols[0].name, "New Name");
        assert_eq!(cols[0].description, Some("Updated desc".to_string()));
    }

    #[test]
    fn delete_collection_test() {
        let conn = setup_test_db();
        let col = create_collection(&conn, "To Delete", None).unwrap();
        delete_collection(&conn, col.id).unwrap();

        let cols = get_collections(&conn).unwrap();
        assert!(cols.is_empty());
    }

    #[test]
    fn create_and_get_folders() {
        let conn = setup_test_db();
        let col = create_collection(&conn, "API", None).unwrap();
        let f1 = create_folder(&conn, col.id, "Auth", None).unwrap();
        let _f2 = create_folder(&conn, col.id, "Users", None).unwrap();
        let _f3 = create_folder(&conn, col.id, "Login", Some(f1.id)).unwrap();

        let folders = get_folders(&conn, col.id).unwrap();
        assert_eq!(folders.len(), 3);
        let names: Vec<&str> = folders.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"Auth"));
        assert!(names.contains(&"Users"));
        assert!(names.contains(&"Login"));

        let login_folder = folders.iter().find(|f| f.name == "Login").unwrap();
        assert_eq!(login_folder.parent_folder_id, Some(f1.id));

        let auth_folder = folders.iter().find(|f| f.name == "Auth").unwrap();
        assert_eq!(auth_folder.parent_folder_id, None);
    }

    #[test]
    fn rename_folder_test() {
        let conn = setup_test_db();
        let col = create_collection(&conn, "API", None).unwrap();
        let folder = create_folder(&conn, col.id, "Old", None).unwrap();
        rename_folder(&conn, folder.id, "New").unwrap();

        let folders = get_folders(&conn, col.id).unwrap();
        assert_eq!(folders[0].name, "New");
    }

    #[test]
    fn delete_folder_cascade() {
        let conn = setup_test_db();
        let col = create_collection(&conn, "API", None).unwrap();
        let folder = create_folder(&conn, col.id, "ToDelete", None).unwrap();
        delete_folder(&conn, folder.id).unwrap();

        let folders = get_folders(&conn, col.id).unwrap();
        assert!(folders.is_empty());
    }

    #[test]
    fn save_and_get_collection_requests() {
        let conn = setup_test_db();
        let col = create_collection(&conn, "API", None).unwrap();
        let headers = vec![("Content-Type".to_string(), "application/json".to_string())];
        let params = vec![("key".to_string(), "value".to_string())];

        save_collection_request(
            &conn,
            &SaveRequestParams {
                collection_id: col.id,
                folder_id: None,
                name: "Get Todos".to_string(),
                method: "GET".to_string(),
                url: "https://jsonplaceholder.typicode.com/todos".to_string(),
                headers: headers.clone(),
                body: None,
                body_type: CollectionBodyType::Text,
                auth_type: CollectionAuthType::None,
                auth_data: None,
                params: params.clone(),
                config_json: None,
                scripts: None,
            },
        )
        .unwrap();
        save_collection_request(
            &conn,
            &SaveRequestParams {
                collection_id: col.id,
                folder_id: None,
                name: "Create Todo".to_string(),
                method: "POST".to_string(),
                url: "https://jsonplaceholder.typicode.com/todos".to_string(),
                headers,
                body: Some(r#"{"title":"test"}"#.to_string()),
                body_type: CollectionBodyType::Text,
                auth_type: CollectionAuthType::Bearer,
                auth_data: Some("token123".to_string()),
                params: vec![],
                config_json: None,
                scripts: None,
            },
        )
        .unwrap();

        let reqs = get_collection_requests(&conn, col.id, None).unwrap();
        assert_eq!(reqs.len(), 2);
        assert_eq!(reqs[0].name, "Get Todos");
        assert_eq!(reqs[1].name, "Create Todo");
        assert_eq!(reqs[0].headers.len(), 1);
        assert_eq!(reqs[1].body, Some(r#"{"title":"test"}"#.to_string()));
        assert_eq!(reqs[1].auth_type, CollectionAuthType::Bearer);
    }

    #[test]
    fn save_request_in_folder() {
        let conn = setup_test_db();
        let col = create_collection(&conn, "API", None).unwrap();
        let folder = create_folder(&conn, col.id, "Auth", None).unwrap();

        save_collection_request(
            &conn,
            &SaveRequestParams {
                collection_id: col.id,
                folder_id: Some(folder.id),
                name: "Login".to_string(),
                method: "POST".to_string(),
                url: "https://api.example.com/login".to_string(),
                headers: vec![],
                body: Some(r#"{"user":"admin"}"#.to_string()),
                body_type: CollectionBodyType::Text,
                auth_type: CollectionAuthType::None,
                auth_data: None,
                params: vec![],
                config_json: None,
                scripts: None,
            },
        )
        .unwrap();

        let root_reqs = get_collection_requests(&conn, col.id, None).unwrap();
        assert!(root_reqs.is_empty());

        let folder_reqs = get_collection_requests(&conn, col.id, Some(folder.id)).unwrap();
        assert_eq!(folder_reqs.len(), 1);
        assert_eq!(folder_reqs[0].name, "Login");
    }

    #[test]
    fn rename_and_move_collection_request() {
        let conn = setup_test_db();
        let col = create_collection(&conn, "API", None).unwrap();
        let folder = create_folder(&conn, col.id, "Folder", None).unwrap();

        let req = save_collection_request(
            &conn,
            &SaveRequestParams::new(col.id, "Old Name", "GET", "https://example.com"),
        )
        .unwrap();

        rename_collection_request(&conn, req.id, "New Name").unwrap();
        move_collection_request(&conn, req.id, Some(folder.id)).unwrap();

        let root_reqs = get_collection_requests(&conn, col.id, None).unwrap();
        assert!(root_reqs.is_empty());

        let folder_reqs = get_collection_requests(&conn, col.id, Some(folder.id)).unwrap();
        assert_eq!(folder_reqs[0].name, "New Name");
    }

    #[test]
    fn delete_collection_request_test() {
        let conn = setup_test_db();
        let col = create_collection(&conn, "API", None).unwrap();
        let req = save_collection_request(
            &conn,
            &SaveRequestParams::new(col.id, "To Delete", "DELETE", "https://example.com/1"),
        )
        .unwrap();

        delete_collection_request(&conn, req.id).unwrap();
        let reqs = get_collection_requests(&conn, col.id, None).unwrap();
        assert!(reqs.is_empty());
    }

    #[test]
    fn save_history_with_request_and_response_data() {
        let conn = setup_test_db();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS request_history (
                id INTEGER PRIMARY KEY,
                method TEXT NOT NULL,
                url TEXT NOT NULL,
                status INTEGER,
                duration_ms INTEGER,
                timestamp TEXT NOT NULL,
                request_data TEXT,
                response_data TEXT
            )",
            [],
        )
        .unwrap();

        let request_json = r#"{"method":"POST","url":"https://api.example.com","headers":[["Content-Type","application/json"]],"body":"{\"name\":\"test\"}"}"#;
        let response_json = r#"{"url":"https://api.example.com","method":"POST","status":201,"headers":[],"body":"{\"id\":1}","duration":150,"size":13,"redirect_chain":[]}"#;

        save_request_history(
            &conn,
            "POST",
            "https://api.example.com",
            Some(201),
            Some(150),
            Some(request_json),
            Some(response_json),
        )
        .unwrap();

        let history = get_request_history(&conn, 10).unwrap();
        assert_eq!(history.len(), 1);
        assert!(history[0].request_data.is_some());
        assert!(history[0].response_data.is_some());
        assert!(history[0].request_data.as_ref().unwrap().contains("POST"));
    }

    #[test]
    fn get_history_entry_by_id_returns_full_data() {
        let conn = setup_test_db();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS request_history (
                id INTEGER PRIMARY KEY,
                method TEXT NOT NULL,
                url TEXT NOT NULL,
                status INTEGER,
                duration_ms INTEGER,
                timestamp TEXT NOT NULL,
                request_data TEXT,
                response_data TEXT
            )",
            [],
        )
        .unwrap();

        let request_json = r#"{"method":"GET","url":"https://example.com"}"#;
        save_request_history(
            &conn,
            "GET",
            "https://example.com",
            Some(200),
            Some(100),
            Some(request_json),
            None,
        )
        .unwrap();

        let entry = get_request_history_entry_by_id(&conn, 1).unwrap();
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.method, "GET");
        assert!(entry.request_data.is_some());
        assert!(entry.response_data.is_none());
    }

    #[test]
    fn get_nonexistent_history_entry_returns_none() {
        let conn = setup_test_db();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS request_history (
                id INTEGER PRIMARY KEY,
                method TEXT NOT NULL,
                url TEXT NOT NULL,
                status INTEGER,
                duration_ms INTEGER,
                timestamp TEXT NOT NULL,
                request_data TEXT,
                response_data TEXT
            )",
            [],
        )
        .unwrap();
        let entry = get_request_history_entry_by_id(&conn, 999).unwrap();
        assert!(entry.is_none());
    }

    #[test]
    fn trim_request_history_removes_oldest() {
        let conn = setup_test_db();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS request_history (
                id INTEGER PRIMARY KEY,
                method TEXT NOT NULL,
                url TEXT NOT NULL,
                status INTEGER,
                duration_ms INTEGER,
                timestamp TEXT NOT NULL,
                request_data TEXT,
                response_data TEXT
            )",
            [],
        )
        .unwrap();

        for i in 0..5 {
            save_request_history(
                &conn,
                "GET",
                &format!("https://example.com/{}", i),
                Some(200),
                Some(100),
                None,
                None,
            )
            .unwrap();
        }

        trim_request_history(&conn, 3).unwrap();
        let history = get_request_history(&conn, 10).unwrap();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].url, "https://example.com/4");
        assert_eq!(history[1].url, "https://example.com/3");
        assert_eq!(history[2].url, "https://example.com/2");
    }

    #[test]
    fn trim_request_history_no_op_when_under_limit() {
        let conn = setup_test_db();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS request_history (
                id INTEGER PRIMARY KEY,
                method TEXT NOT NULL,
                url TEXT NOT NULL,
                status INTEGER,
                duration_ms INTEGER,
                timestamp TEXT NOT NULL,
                request_data TEXT,
                response_data TEXT
            )",
            [],
        )
        .unwrap();

        for i in 0..3 {
            save_request_history(
                &conn,
                "GET",
                &format!("https://example.com/{}", i),
                Some(200),
                Some(100),
                None,
                None,
            )
            .unwrap();
        }

        trim_request_history(&conn, 5).unwrap();
        let history = get_request_history(&conn, 10).unwrap();
        assert_eq!(history.len(), 3);
    }
}
