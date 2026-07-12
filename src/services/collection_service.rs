use crate::error::AppError;
use crate::persistence::database::{
    self, Collection, CollectionFolder, CollectionRequest, SaveRequestParams,
};
use rusqlite::Connection;

pub fn get_all(conn: &Connection) -> Result<Vec<Collection>, AppError> {
    Ok(database::get_collections(conn)?)
}

pub fn create(conn: &Connection, name: &str) -> Result<Collection, AppError> {
    Ok(database::create_collection(conn, name, None)?)
}

pub fn update(conn: &Connection, collection: &Collection) -> Result<(), AppError> {
    Ok(database::update_collection(conn, collection)?)
}

pub fn delete(conn: &Connection, id: i32) -> Result<(), AppError> {
    Ok(database::delete_collection(conn, id)?)
}

pub fn create_and_refresh(conn: &Connection, name: &str) -> Result<Vec<Collection>, AppError> {
    create(conn, name)?;
    get_all(conn)
}

pub fn delete_and_refresh(conn: &Connection, id: i32) -> Result<Vec<Collection>, AppError> {
    delete(conn, id)?;
    get_all(conn)
}

pub fn rename(conn: &Connection, collection: &Collection, new_name: &str) -> Result<(), AppError> {
    let mut updated = collection.clone();
    updated.name = new_name.to_string();
    update(conn, &updated)
}

pub fn get_folders(conn: &Connection, collection_id: i32) -> Result<Vec<CollectionFolder>, AppError> {
    Ok(database::get_folders(conn, collection_id)?)
}

pub fn create_folder(
    conn: &Connection,
    collection_id: i32,
    name: &str,
) -> Result<CollectionFolder, AppError> {
    Ok(database::create_folder(conn, collection_id, name, None)?)
}

pub fn delete_folder(conn: &Connection, id: i32) -> Result<(), AppError> {
    Ok(database::delete_folder(conn, id)?)
}

pub fn rename_folder(conn: &Connection, id: i32, new_name: &str) -> Result<(), AppError> {
    Ok(database::rename_folder(conn, id, new_name)?)
}

pub fn create_folder_with_parent(
    conn: &Connection,
    collection_id: i32,
    name: &str,
    parent_folder_id: Option<i32>,
) -> Result<Vec<CollectionFolder>, AppError> {
    database::create_folder(conn, collection_id, name, parent_folder_id)?;
    get_folders(conn, collection_id)
}

pub fn delete_folder_and_refresh(
    conn: &Connection,
    collection_id: i32,
    folder_id: i32,
) -> Result<Vec<CollectionFolder>, AppError> {
    delete_folder(conn, folder_id)?;
    get_folders(conn, collection_id)
}

pub fn get_requests(
    conn: &Connection,
    collection_id: i32,
    folder_id: Option<i32>,
) -> Result<Vec<CollectionRequest>, AppError> {
    Ok(database::get_collection_requests(conn, collection_id, folder_id)?)
}

pub fn save_request(
    conn: &Connection,
    params: &SaveRequestParams,
) -> Result<CollectionRequest, AppError> {
    Ok(database::save_collection_request(conn, params)?)
}

pub fn rename_request(conn: &Connection, id: i32, new_name: &str) -> Result<(), AppError> {
    Ok(database::rename_collection_request(conn, id, new_name)?)
}

#[allow(dead_code)]
pub fn move_request(
    conn: &Connection,
    id: i32,
    new_folder_id: Option<i32>,
) -> Result<(), AppError> {
    Ok(database::move_collection_request(conn, id, new_folder_id)?)
}

pub fn delete_request(conn: &Connection, id: i32) -> Result<(), AppError> {
    Ok(database::delete_collection_request(conn, id)?)
}

pub fn delete_request_and_refresh(
    conn: &Connection,
    collection_id: i32,
    folder_id: Option<i32>,
    request_id: i32,
) -> Result<Vec<CollectionRequest>, AppError> {
    delete_request(conn, request_id)?;
    get_requests(conn, collection_id, folder_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
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
    fn create_and_list_collection() {
        let conn = setup_test_db();
        let col = create(&conn, "My API").unwrap();
        assert_eq!(col.name, "My API");

        let cols = get_all(&conn).unwrap();
        assert_eq!(cols.len(), 1);
    }

    #[test]
    fn create_and_list_folder() {
        let conn = setup_test_db();
        let col = create(&conn, "API").unwrap();
        let folder = create_folder(&conn, col.id, "Auth").unwrap();
        assert_eq!(folder.name, "Auth");

        let folders = get_folders(&conn, col.id).unwrap();
        assert_eq!(folders.len(), 1);
    }

    #[test]
    fn save_and_get_request() {
        let conn = setup_test_db();
        let col = create(&conn, "API").unwrap();
        let req = save_request(
            &conn,
            &SaveRequestParams::new(col.id, "Get Todos", "GET", "https://api.example.com/todos"),
        )
        .unwrap();
        assert_eq!(req.name, "Get Todos");

        let reqs = get_requests(&conn, col.id, None).unwrap();
        assert_eq!(reqs.len(), 1);
    }

    #[test]
    fn rename_collection_test() {
        let conn = setup_test_db();
        let col = create(&conn, "Old").unwrap();
        rename(&conn, &col, "New").unwrap();

        let cols = get_all(&conn).unwrap();
        assert_eq!(cols[0].name, "New");
    }

    #[test]
    fn rename_folder_test() {
        let conn = setup_test_db();
        let col = create(&conn, "API").unwrap();
        let folder = create_folder(&conn, col.id, "Old").unwrap();
        rename_folder(&conn, folder.id, "New").unwrap();

        let folders = get_folders(&conn, col.id).unwrap();
        assert_eq!(folders[0].name, "New");
    }

    #[test]
    fn rename_request_test() {
        let conn = setup_test_db();
        let col = create(&conn, "API").unwrap();
        let req = save_request(
            &conn,
            &SaveRequestParams::new(col.id, "Old", "GET", "https://example.com"),
        )
        .unwrap();
        rename_request(&conn, req.id, "New").unwrap();

        let reqs = get_requests(&conn, col.id, None).unwrap();
        assert_eq!(reqs[0].name, "New");
    }

    #[test]
    fn delete_request_test() {
        let conn = setup_test_db();
        let col = create(&conn, "API").unwrap();
        let req = save_request(
            &conn,
            &SaveRequestParams::new(col.id, "To Delete", "DELETE", "https://example.com/1"),
        )
        .unwrap();

        delete_request(&conn, req.id).unwrap();
        let reqs = get_requests(&conn, col.id, None).unwrap();
        assert!(reqs.is_empty());
    }

    #[test]
    fn move_request_test() {
        let conn = setup_test_db();
        let col = create(&conn, "API").unwrap();
        let folder = create_folder(&conn, col.id, "Auth").unwrap();
        let req = save_request(
            &conn,
            &SaveRequestParams::new(col.id, "Login", "POST", "https://api.example.com/login"),
        )
        .unwrap();

        move_request(&conn, req.id, Some(folder.id)).unwrap();
        let root_reqs = get_requests(&conn, col.id, None).unwrap();
        assert!(root_reqs.is_empty());
        let folder_reqs = get_requests(&conn, col.id, Some(folder.id)).unwrap();
        assert_eq!(folder_reqs.len(), 1);
    }

    #[test]
    fn create_and_refresh_returns_full_list() {
        let conn = setup_test_db();
        let cols = create_and_refresh(&conn, "API v1").unwrap();
        assert_eq!(cols.len(), 1);

        let cols = create_and_refresh(&conn, "API v2").unwrap();
        assert_eq!(cols.len(), 2);
    }

    #[test]
    fn delete_and_refresh_removes_entry() {
        let conn = setup_test_db();
        let col = create(&conn, "API").unwrap();
        create(&conn, "Auth").unwrap();
        let cols = delete_and_refresh(&conn, col.id).unwrap();
        assert_eq!(cols.len(), 1);
    }

    #[test]
    fn create_folder_with_parent_returns_folders() {
        let conn = setup_test_db();
        let col = create(&conn, "API").unwrap();
        let folders = create_folder_with_parent(&conn, col.id, "Auth", None).unwrap();
        assert_eq!(folders.len(), 1);
    }

    #[test]
    fn delete_folder_and_refresh_removes_folder() {
        let conn = setup_test_db();
        let col = create(&conn, "API").unwrap();
        let folder = create_folder(&conn, col.id, "ToDelete").unwrap();
        let folders = delete_folder_and_refresh(&conn, col.id, folder.id).unwrap();
        assert!(folders.is_empty());
    }

    #[test]
    fn delete_request_and_refresh_removes_request() {
        let conn = setup_test_db();
        let col = create(&conn, "API").unwrap();
        let req = save_request(
            &conn,
            &SaveRequestParams::new(col.id, "To Delete", "DELETE", "https://example.com/1"),
        )
        .unwrap();
        let reqs = delete_request_and_refresh(&conn, col.id, None, req.id).unwrap();
        assert!(reqs.is_empty());
    }
}
