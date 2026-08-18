use crate::error::AppError;
use rusqlite::{params, Connection, Result};

use super::{Collection, CollectionFolder, CollectionRequest, SaveRequestParams};

pub fn create_collection(
    conn: &Connection,
    name: &str,
    description: Option<&str>,
) -> Result<Collection> {
    let variables: Vec<(String, String)> = Vec::new();
    let variables_json = serde_json::to_value(&variables)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
    conn.execute(
        "INSERT INTO collections (name, description, variables) VALUES (?1, ?2, ?3)",
        params![name, description, &variables_json.to_string()],
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
        description: description.map(std::string::ToString::to_string),
        sort_order: max_order + 1,
        variables,
    })
}

pub fn get_collections(conn: &Connection) -> Result<Vec<Collection>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, sort_order, variables FROM collections ORDER BY sort_order, name",
    )?;
    let rows = stmt.query_map([], |row| {
        let variables_json: String = row.get(4)?;
        let variables: Vec<(String, String)> =
            serde_json::from_str(&variables_json).unwrap_or_default();
        Ok(Collection {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            sort_order: row.get(3)?,
            variables,
        })
    })?;
    rows.collect()
}

pub fn update_collection(conn: &Connection, collection: &Collection) -> Result<()> {
    let variables_json = serde_json::to_value(&collection.variables)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
    conn.execute(
        "UPDATE collections SET name = ?1, description = ?2, variables = ?3 WHERE id = ?4",
        params![
            collection.name,
            collection.description,
            &variables_json.to_string(),
            collection.id,
        ],
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

#[allow(dead_code)]
pub fn get_collection_request_by_id(
    conn: &Connection,
    id: i32,
) -> Result<Option<CollectionRequest>> {
    let mut stmt = conn.prepare(
        "SELECT id, collection_id, folder_id, name, method, url, headers, body, body_type, auth_type, auth_data, params, config_json, scripts, sort_order FROM collection_requests WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map([id], parse_collection_request)?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

#[allow(dead_code)]
pub fn get_collection_by_id(conn: &Connection, id: i32) -> Result<Option<Collection>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, sort_order, variables FROM collections WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map([id], |row| {
        let variables_json: String = row.get(4)?;
        let variables: Vec<(String, String)> =
            serde_json::from_str(&variables_json).unwrap_or_default();
        Ok(Collection {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            sort_order: row.get(3)?,
            variables,
        })
    })?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

#[allow(dead_code)]
pub fn get_collection_by_name(conn: &Connection, name: &str) -> Result<Option<Collection>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, sort_order, variables FROM collections WHERE name = ?1",
    )?;
    let mut rows = stmt.query_map([name], |row| {
        let variables_json: String = row.get(4)?;
        let variables: Vec<(String, String)> =
            serde_json::from_str(&variables_json).unwrap_or_default();
        Ok(Collection {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            sort_order: row.get(3)?,
            variables,
        })
    })?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn rename_collection_request(conn: &Connection, id: i32, new_name: &str) -> Result<()> {
    conn.execute(
        "UPDATE collection_requests SET name = ?1 WHERE id = ?2",
        params![new_name, id],
    )?;
    Ok(())
}

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

pub fn swap_request_sort_order(conn: &Connection, id_a: i32, id_b: i32) -> Result<()> {
    let order_a: i32 = conn
        .query_row(
            "SELECT sort_order FROM collection_requests WHERE id = ?1",
            [id_a],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let order_b: i32 = conn
        .query_row(
            "SELECT sort_order FROM collection_requests WHERE id = ?1",
            [id_b],
            |row| row.get(0),
        )
        .unwrap_or(0);
    conn.execute(
        "UPDATE collection_requests SET sort_order = ?1 WHERE id = ?2",
        params![order_b, id_a],
    )?;
    conn.execute(
        "UPDATE collection_requests SET sort_order = ?1 WHERE id = ?2",
        params![order_a, id_b],
    )?;
    Ok(())
}

pub fn get_adjacent_requests(
    conn: &Connection,
    collection_id: i32,
    folder_id: Option<i32>,
    current_sort_order: i32,
) -> Result<(Option<i32>, Option<i32>), AppError> {
    let prev: Option<i32> = conn
        .query_row(
            "SELECT id FROM collection_requests WHERE collection_id = ?1 AND folder_id IS ?2 AND sort_order < ?3 ORDER BY sort_order DESC LIMIT 1",
            params![collection_id, folder_id, current_sort_order],
            |row| row.get(0),
        )
        .ok();
    let next: Option<i32> = conn
        .query_row(
            "SELECT id FROM collection_requests WHERE collection_id = ?1 AND folder_id IS ?2 AND sort_order > ?3 ORDER BY sort_order ASC LIMIT 1",
            params![collection_id, folder_id, current_sort_order],
            |row| row.get(0),
        )
        .ok();
    Ok((prev, next))
}
