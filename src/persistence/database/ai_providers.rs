use crate::ai::types::AiProviderConfig;
use rusqlite::{params, Connection, Result};

pub fn get_all_ai_providers(conn: &Connection) -> Result<Vec<AiProviderConfig>> {
    let mut stmt = conn.prepare(
        "SELECT id, provider, name, base_url, model, max_tokens, temperature, is_default
         FROM ai_provider_configs ORDER BY is_default DESC, name ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        let provider_str: String = row.get(1)?;
        let provider = match provider_str.as_str() {
            "OpenAi" => crate::ai::types::AiProvider::OpenAi,
            "Anthropic" => crate::ai::types::AiProvider::Anthropic,
            "Ollama" => crate::ai::types::AiProvider::Ollama,
            _ => crate::ai::types::AiProvider::Custom,
        };
        Ok(AiProviderConfig {
            id: Some(row.get(0)?),
            provider,
            name: row.get(2)?,
            base_url: row.get(3)?,
            model: row.get(4)?,
            max_tokens: row.get::<_, i64>(5)? as u32,
            temperature: row.get(6)?,
            is_default: row.get::<_, i64>(7)? != 0,
        })
    })?;
    rows.collect()
}

pub fn insert_ai_provider(conn: &Connection, config: &AiProviderConfig) -> Result<i32> {
    let now = chrono::Utc::now().to_rfc3339();
    let provider_str = config.provider.to_string();
    let mut stmt = conn.prepare(
        "INSERT INTO ai_provider_configs (provider, name, base_url, model, max_tokens, temperature, is_default, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )?;
    stmt.execute(params![
        provider_str,
        config.name,
        config.base_url,
        config.model,
        config.max_tokens as i64,
        config.temperature,
        config.is_default as i64,
        now,
        now,
    ])?;
    Ok(conn.last_insert_rowid() as i32)
}

pub fn update_ai_provider(conn: &Connection, id: i32, config: &AiProviderConfig) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    let provider_str = config.provider.to_string();
    conn.execute(
        "UPDATE ai_provider_configs SET provider=?1, name=?2, base_url=?3, model=?4,
         max_tokens=?5, temperature=?6, is_default=?7, updated_at=?8 WHERE id=?9",
        params![
            provider_str,
            config.name,
            config.base_url,
            config.model,
            config.max_tokens as i64,
            config.temperature,
            config.is_default as i64,
            now,
            id,
        ],
    )?;
    Ok(())
}

pub fn delete_ai_provider(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM ai_provider_configs WHERE id = ?1", [id])?;
    Ok(())
}
