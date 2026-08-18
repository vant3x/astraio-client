use rusqlite::{params, Connection, Result};

pub fn create_mock_server(conn: &Connection, name: &str, port: u16) -> Result<i32> {
    conn.execute(
        "INSERT INTO mock_servers (name, port) VALUES (?1, ?2)",
        params![name, i64::from(port)],
    )?;
    Ok(conn.last_insert_rowid() as i32)
}

pub fn get_all_mock_servers(
    conn: &Connection,
) -> Result<Vec<crate::protocols::mock_server::MockServerConfig>> {
    let mut stmt =
        conn.prepare("SELECT id, name, port, host, enabled FROM mock_servers ORDER BY id ASC")?;
    let servers = stmt.query_map([], |row| {
        let id: i32 = row.get(0)?;
        let name: String = row.get(1)?;
        let port: i64 = row.get(2)?;
        let host: String = row.get(3)?;
        let enabled: bool = row.get::<_, i64>(4)? != 0;
        Ok((id, name, port as u16, host, enabled))
    })?;

    let mut result = Vec::new();
    for server_row in servers {
        let (id, name, port, host, enabled) = server_row?;
        let endpoints = get_mock_endpoints(conn, id)?;
        result.push(crate::protocols::mock_server::MockServerConfig {
            id,
            name,
            port,
            host,
            enabled,
            endpoints,
        });
    }
    Ok(result)
}

pub fn get_mock_server(
    conn: &Connection,
    id: i32,
) -> Result<crate::protocols::mock_server::MockServerConfig> {
    let (name, port, host, enabled) = conn.query_row(
        "SELECT name, port, host, enabled FROM mock_servers WHERE id = ?1",
        [id],
        |row| {
            let name: String = row.get(0)?;
            let port: i64 = row.get(1)?;
            let host: String = row.get(2)?;
            let enabled: bool = row.get::<_, i64>(3)? != 0;
            Ok((name, port as u16, host, enabled))
        },
    )?;
    let endpoints = get_mock_endpoints(conn, id)?;
    Ok(crate::protocols::mock_server::MockServerConfig {
        id,
        name,
        port,
        host,
        enabled,
        endpoints,
    })
}

#[allow(dead_code)]
pub fn update_mock_server(
    conn: &Connection,
    id: i32,
    name: &str,
    port: u16,
    enabled: bool,
) -> Result<()> {
    conn.execute(
        "UPDATE mock_servers SET name = ?1, port = ?2, enabled = ?3 WHERE id = ?4",
        params![name, i64::from(port), i64::from(enabled), id],
    )?;
    Ok(())
}

pub fn delete_mock_server(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM mock_endpoints WHERE mock_server_id = ?1", [id])?;
    conn.execute("DELETE FROM mock_servers WHERE id = ?1", [id])?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn create_mock_endpoint(
    conn: &Connection,
    mock_server_id: i32,
    method: &str,
    path: &str,
    status: u16,
    headers: &[(String, String)],
    body: Option<&str>,
    delay_ms: u64,
) -> Result<i32> {
    let headers_json = serde_json::to_string(headers).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "INSERT INTO mock_endpoints (mock_server_id, method, path, status, headers, body, delay_ms) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![mock_server_id, method, path, i64::from(status), headers_json, body, delay_ms as i64],
    )?;
    Ok(conn.last_insert_rowid() as i32)
}

pub fn get_mock_endpoints(
    conn: &Connection,
    mock_server_id: i32,
) -> Result<Vec<crate::protocols::mock_server::MockEndpoint>> {
    let mut stmt = conn.prepare(
        "SELECT id, mock_server_id, method, path, status, headers, body, delay_ms, sort_order FROM mock_endpoints WHERE mock_server_id = ?1 ORDER BY sort_order ASC, id ASC",
    )?;
    let endpoints = stmt.query_map([mock_server_id], |row| {
        let headers_json: String = row.get(5)?;
        let headers: Vec<(String, String)> =
            serde_json::from_str(&headers_json).unwrap_or_default();
        Ok(crate::protocols::mock_server::MockEndpoint {
            id: row.get(0)?,
            mock_server_id: row.get(1)?,
            method: row.get(2)?,
            path: row.get(3)?,
            status: row.get::<_, i64>(4)? as u16,
            headers,
            body: row.get(6)?,
            delay_ms: row.get::<_, i64>(7)? as u64,
            sort_order: row.get::<_, i64>(8)? as i32,
        })
    })?;

    let mut result = Vec::new();
    for ep in endpoints {
        result.push(ep?);
    }
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
pub fn update_mock_endpoint(
    conn: &Connection,
    id: i32,
    method: &str,
    path: &str,
    status: u16,
    headers: &[(String, String)],
    body: Option<&str>,
    delay_ms: u64,
) -> Result<()> {
    let headers_json = serde_json::to_string(headers).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "UPDATE mock_endpoints SET method = ?1, path = ?2, status = ?3, headers = ?4, body = ?5, delay_ms = ?6 WHERE id = ?7",
        params![method, path, i64::from(status), headers_json, body, delay_ms as i64, id],
    )?;
    Ok(())
}

pub fn delete_mock_endpoint(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM mock_endpoints WHERE id = ?1", [id])?;
    Ok(())
}
