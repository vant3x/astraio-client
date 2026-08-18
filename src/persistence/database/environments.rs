use rusqlite::{params, Connection, Result};

use super::Environment;

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
            &env.id,
        ],
    )?;
    Ok(())
}

pub fn delete_environment(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM environments WHERE id = ?1", [id])?;
    Ok(())
}
