use crate::error::AppError;
use crate::utils::timestamp_seconds;
use rusqlite::{params, Connection};

use super::Session;

pub fn save_session(conn: &Connection, session: &Session) -> std::result::Result<(), AppError> {
    conn.execute(
        "INSERT OR REPLACE INTO sessions (id, name, cookies_json, headers_json, auth_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            session.id,
            session.name,
            session.cookies_json,
            session.headers_json,
            session.auth_json,
            session.created_at,
            session.updated_at,
        ],
    )
    .map_err(|e| AppError::Database(format!("Failed to save session: {e}")))?;
    Ok(())
}

pub fn load_sessions(conn: &Connection) -> std::result::Result<Vec<Session>, AppError> {
    let mut stmt = conn
        .prepare("SELECT id, name, cookies_json, headers_json, auth_json, created_at, updated_at FROM sessions ORDER BY updated_at DESC")
        .map_err(|e| AppError::Database(format!("Failed to prepare sessions query: {e}")))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Session {
                id: row.get(0)?,
                name: row.get(1)?,
                cookies_json: row.get(2)?,
                headers_json: row.get(3)?,
                auth_json: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| AppError::Database(format!("Failed to query sessions: {e}")))?;

    let mut sessions = Vec::new();
    for s in rows.flatten() {
        sessions.push(s);
    }
    Ok(sessions)
}

pub fn delete_session(conn: &Connection, id: &str) -> std::result::Result<(), AppError> {
    conn.execute("DELETE FROM sessions WHERE id = ?1", params![id])
        .map_err(|e| AppError::Database(format!("Failed to delete session: {e}")))?;
    Ok(())
}

pub fn rename_session(
    conn: &Connection,
    id: &str,
    new_name: &str,
) -> std::result::Result<(), AppError> {
    let now = timestamp_seconds();
    conn.execute(
        "UPDATE sessions SET name = ?1, updated_at = ?2 WHERE id = ?3",
        params![new_name, now, id],
    )
    .map_err(|e| AppError::Database(format!("Failed to rename session: {e}")))?;
    Ok(())
}
