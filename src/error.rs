use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Database error: {0}")]
    DatabaseSource(#[source] Arc<rusqlite::Error>),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("HTTP transport error: {0}")]
    HttpTransport(#[source] Arc<reqwest::Error>),

    #[error("IO error: {0}")]
    Io(String),

    #[error("IO error: {0}")]
    IoSource(#[source] Arc<std::io::Error>),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Serialization error: {0}")]
    SerializationSource(#[source] Arc<serde_json::Error>),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("OAuth2 error: {0}")]
    OAuth2(String),
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::DatabaseSource(Arc::new(err))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::HttpTransport(Arc::new(err))
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoSource(Arc::new(err))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::SerializationSource(Arc::new(err))
    }
}

impl From<http::method::InvalidMethod> for AppError {
    fn from(err: http::method::InvalidMethod) -> Self {
        AppError::Http(err.to_string())
    }
}
