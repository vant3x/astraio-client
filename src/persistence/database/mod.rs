use crate::error::AppError;
use directories::ProjectDirs;
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

// ── Sub-modules ─────────────────────────────────────────────────────────────

mod ai_providers;
mod collections;
mod cookies;
mod environments;
mod history;
mod mock_servers;
mod sessions;
mod settings;

// ── Re-exports (backward compat: crate::persistence::database::func still works) ──

pub use ai_providers::*;
pub use collections::*;
pub use cookies::*;
pub use environments::*;
pub use history::*;
pub use mock_servers::*;
pub use sessions::*;
pub use settings::*;

// ── Types ───────────────────────────────────────────────────────────────────

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
    #[serde(default)]
    pub variables: Vec<(String, String)>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub cookies_json: String,
    pub headers_json: String,
    pub auth_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl std::fmt::Display for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
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
        scripts: Option<&str>,
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
            scripts: scripts.map(str::to_string),
        }
    }
}

// ── Schema & Init ───────────────────────────────────────────────────────────

fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
    let pragma = format!("PRAGMA table_info({table})");
    if let Ok(mut stmt) = conn.prepare(&pragma) {
        let cols: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default();
        cols.contains(&column.to_string())
    } else {
        false
    }
}

fn add_column_if_missing(
    conn: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> std::result::Result<(), AppError> {
    if !column_exists(conn, table, column) {
        conn.execute(
            &format!("ALTER TABLE {table} ADD COLUMN {column} {definition}"),
            [],
        )?;
    }
    Ok(())
}

fn get_db_path() -> std::result::Result<PathBuf, AppError> {
    let proj_dirs = ProjectDirs::from("com", "astraio", "client")
        .ok_or_else(|| AppError::Database("Failed to determine project directories".to_string()))?;
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir)
        .map_err(|e| AppError::Io(format!("Failed to create data directory: {e}")))?;
    Ok(data_dir.join("astraio.db"))
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
    add_column_if_missing(conn, "environments", "default_endpoint", "TEXT")?;
    add_column_if_missing(conn, "environments", "secret_keys", "TEXT NOT NULL DEFAULT '[]'")?;
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
    add_column_if_missing(conn, "request_history", "request_data", "TEXT")?;
    add_column_if_missing(conn, "request_history", "response_data", "TEXT")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS collections (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            sort_order INTEGER NOT NULL DEFAULT 0,
            variables TEXT NOT NULL DEFAULT '[]'
        )",
        [],
    )?;
    add_column_if_missing(conn, "collections", "variables", "TEXT NOT NULL DEFAULT '[]'")?;
    add_column_if_missing(conn, "collections", "sort_order", "INTEGER NOT NULL DEFAULT 0")?;
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
    add_column_if_missing(conn, "collection_folders", "sort_order", "INTEGER NOT NULL DEFAULT 0")?;
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
    add_column_if_missing(conn, "collection_requests", "scripts", "TEXT")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_collection_folders_collection_id ON collection_folders(collection_id)",
        [],
    )
    .ok();
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_collection_folders_parent ON collection_folders(parent_folder_id)",
        [],
    )
    .ok();
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_collection_requests_collection ON collection_requests(collection_id, folder_id)",
        [],
    )
    .ok();
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_collection_requests_sort ON collection_requests(collection_id, folder_id, sort_order)",
        [],
    )
    .ok();
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_request_history_timestamp ON request_history(timestamp)",
        [],
    )
    .ok();
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_request_history_url ON request_history(url)",
        [],
    )
    .ok();

    // FTS5 virtual table for fast full-text search on request history
    conn.execute_batch(
        "CREATE VIRTUAL TABLE IF NOT EXISTS request_history_fts USING fts5(
            url, method, request_data, response_data,
            content='request_history',
            content_rowid='id'
        );

        -- Triggers to keep FTS index in sync with the main table
        CREATE TRIGGER IF NOT EXISTS request_history_ai AFTER INSERT ON request_history BEGIN
            INSERT INTO request_history_fts(rowid, url, method, request_data, response_data)
            VALUES (new.id, new.url, new.method, new.request_data, new.response_data);
        END;

        CREATE TRIGGER IF NOT EXISTS request_history_ad AFTER DELETE ON request_history BEGIN
            INSERT INTO request_history_fts(request_history_fts, rowid, url, method, request_data, response_data)
            VALUES ('delete', old.id, old.url, old.method, old.request_data, old.response_data);
        END;

        CREATE TRIGGER IF NOT EXISTS request_history_au AFTER UPDATE ON request_history BEGIN
            INSERT INTO request_history_fts(request_history_fts, rowid, url, method, request_data, response_data)
            VALUES ('delete', old.id, old.url, old.method, old.request_data, old.response_data);
            INSERT INTO request_history_fts(rowid, url, method, request_data, response_data)
            VALUES (new.id, new.url, new.method, new.request_data, new.response_data);
        END;"
    )
    .ok();

    // Cookies table — idempotent migration
    conn.execute(
        "CREATE TABLE IF NOT EXISTS cookies (
            id INTEGER PRIMARY KEY,
            domain TEXT NOT NULL,
            name TEXT NOT NULL,
            value TEXT NOT NULL,
            path TEXT NOT NULL DEFAULT '/',
            expires_at TEXT,
            secure INTEGER NOT NULL DEFAULT 0,
            http_only INTEGER NOT NULL DEFAULT 0,
            same_site TEXT NOT NULL DEFAULT 'Lax',
            created_at TEXT NOT NULL
        )",
        [],
    )
    .ok();
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_cookies_domain_name_path ON cookies(domain, name, path)",
        [],
    )
    .ok();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            cookies_json TEXT NOT NULL DEFAULT '[]',
            headers_json TEXT NOT NULL DEFAULT '[]',
            auth_json TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )
    .ok();

    // Mock servers
    conn.execute(
        "CREATE TABLE IF NOT EXISTS mock_servers (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            port INTEGER NOT NULL DEFAULT 3001,
            host TEXT NOT NULL DEFAULT '127.0.0.1',
            enabled INTEGER NOT NULL DEFAULT 1
        )",
        [],
    )
    .ok();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS mock_endpoints (
            id INTEGER PRIMARY KEY,
            mock_server_id INTEGER NOT NULL,
            method TEXT NOT NULL DEFAULT 'GET',
            path TEXT NOT NULL DEFAULT '/',
            status INTEGER NOT NULL DEFAULT 200,
            headers TEXT NOT NULL DEFAULT '[]',
            body TEXT,
            delay_ms INTEGER NOT NULL DEFAULT 0,
            sort_order INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (mock_server_id) REFERENCES mock_servers(id) ON DELETE CASCADE
        )",
        [],
    )
    .ok();

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_mock_endpoints_server ON mock_endpoints(mock_server_id)",
        [],
    )
    .ok();

    // AI provider configs
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ai_provider_configs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            provider TEXT NOT NULL,
            name TEXT NOT NULL,
            base_url TEXT NOT NULL,
            model TEXT NOT NULL,
            max_tokens INTEGER NOT NULL DEFAULT 4096,
            temperature REAL NOT NULL DEFAULT 0.7,
            is_default INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )
    .ok();

    // AI conversations
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ai_conversations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            provider_config_id INTEGER NOT NULL,
            title TEXT,
            system_prompt TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (provider_config_id) REFERENCES ai_provider_configs(id) ON DELETE CASCADE
        )",
        [],
    )
    .ok();

    // AI messages
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ai_messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            conversation_id INTEGER NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            tokens_used INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            FOREIGN KEY (conversation_id) REFERENCES ai_conversations(id) ON DELETE CASCADE
        )",
        [],
    )
    .ok();

    Ok(())
}

pub fn init() -> std::result::Result<Connection, AppError> {
    let db_path = get_db_path()?;
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .map_err(|e| AppError::Database(format!("Failed to set pragmas: {e}")))?;
    init_schema(&conn)?;
    // Rebuild FTS index for any existing rows (handles upgrades)
    conn.execute_batch("INSERT INTO request_history_fts(request_history_fts) VALUES('rebuild');")
        .ok();
    Ok(conn)
}

// ── Tests ───────────────────────────────────────────────────────────────────

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
                sort_order INTEGER NOT NULL DEFAULT 0,
                variables TEXT NOT NULL DEFAULT '[]'
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
                &format!("https://example.com/{i}"),
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
                &format!("https://example.com/{i}"),
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
                &format!("https://example.com/{i}"),
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
