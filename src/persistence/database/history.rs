use crate::utils::timestamp_seconds;
use rusqlite::{params, Connection, Result};

use super::RequestHistoryEntry;

pub const DEFAULT_HISTORY_LIMIT: usize = 500;
pub const MAX_HISTORY_RESPONSE_BYTES: usize = 50 * 1024;

pub fn save_request_history(
    conn: &Connection,
    method: &str,
    url: &str,
    status: Option<u16>,
    duration_ms: Option<u64>,
    request_data: Option<&str>,
    response_data: Option<&str>,
) -> Result<()> {
    let timestamp = timestamp_seconds();
    conn.execute(
        "INSERT INTO request_history (method, url, status, duration_ms, timestamp, request_data, response_data) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![method, url, status.map(i64::from), duration_ms.map(|d| d as i64), timestamp, request_data, response_data],
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

fn sanitize_fts5_query(query: &str) -> String {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    // Wrap in double quotes to force literal phrase search.
    // Escape internal double quotes by doubling them per FTS5 rules.
    let escaped = trimmed.replace('"', "\"\"");
    format!("\"{}\"*", escaped)
}

pub fn search_request_history(
    conn: &Connection,
    query: &str,
    method_filter: &str,
    limit: usize,
) -> Result<Vec<RequestHistoryEntry>> {
    let has_query = !query.is_empty();
    let has_method = !method_filter.is_empty();

    if has_query {
        // Use FTS5 for full-text search when there's a search query
        let fts_pattern = sanitize_fts5_query(query);
        let sql = if has_method {
            "SELECT h.id, h.method, h.url, h.status, h.duration_ms, h.timestamp, h.request_data, h.response_data
             FROM request_history h
             INNER JOIN request_history_fts fts ON h.id = fts.rowid
             WHERE request_history_fts MATCH ?1 AND h.method LIKE ?2
             ORDER BY h.id DESC LIMIT ?3"
        } else {
            "SELECT h.id, h.method, h.url, h.status, h.duration_ms, h.timestamp, h.request_data, h.response_data
             FROM request_history h
             INNER JOIN request_history_fts fts ON h.id = fts.rowid
             WHERE request_history_fts MATCH ?1
             ORDER BY h.id DESC LIMIT ?2"
        };

        let method_pattern = format!("%{method_filter}%");
        let limit_val = limit as i64;

        let mut stmt = conn.prepare(sql)?;
        let entries: Vec<RequestHistoryEntry> = if has_method {
            stmt.query_and_then(
                rusqlite::params![fts_pattern, method_pattern, limit_val],
                map_history_row,
            )?
            .collect::<Result<Vec<_>, _>>()?
        } else {
            stmt.query_and_then(rusqlite::params![fts_pattern, limit_val], map_history_row)?
                .collect::<Result<Vec<_>, _>>()?
        };
        Ok(entries)
    } else {
        // Fall back to LIKE for method-only filtering (no FTS needed)
        let sql = if has_method {
            "SELECT id, method, url, status, duration_ms, timestamp, request_data, response_data FROM request_history WHERE method LIKE ?1 ORDER BY id DESC LIMIT ?2"
        } else {
            "SELECT id, method, url, status, duration_ms, timestamp, request_data, response_data FROM request_history ORDER BY id DESC LIMIT ?1"
        };

        let method_pattern = format!("%{method_filter}%");
        let limit_val = limit as i64;

        let mut stmt = conn.prepare(sql)?;
        let entries: Vec<RequestHistoryEntry> = if has_method {
            stmt.query_and_then(
                rusqlite::params![method_pattern, limit_val],
                map_history_row,
            )?
            .collect::<Result<Vec<_>, _>>()?
        } else {
            stmt.query_and_then(rusqlite::params![limit_val], map_history_row)?
                .collect::<Result<Vec<_>, _>>()?
        };
        Ok(entries)
    }
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
