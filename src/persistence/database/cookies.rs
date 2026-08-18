use crate::error::AppError;
use crate::utils::timestamp_seconds;
use rusqlite::{params, Connection};

/// Persist a single cookie to `SQLite` (efficient for response handlers).
#[allow(dead_code)]
pub fn save_cookie(
    conn: &Connection,
    cookie: &crate::cookie::Cookie,
) -> std::result::Result<(), AppError> {
    let now = timestamp_seconds();
    conn.execute(
        "INSERT OR REPLACE INTO cookies
            (domain, name, value, path, expires_at, secure, http_only, same_site, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            cookie.domain,
            cookie.name,
            cookie.value,
            cookie.path,
            cookie.expires,
            i32::from(cookie.secure),
            i32::from(cookie.http_only),
            cookie.same_site.to_string(),
            now,
        ],
    )
    .map_err(|e| AppError::Database(format!("Failed to save cookie: {e}")))?;
    Ok(())
}

/// Persist the entire `CookieJar` to `SQLite`.
/// Uses a single transaction for performance.
pub fn save_cookies(
    conn: &Connection,
    jar: &crate::cookie::CookieJar,
) -> std::result::Result<(), AppError> {
    let now = timestamp_seconds();
    let all_domains: Vec<String> = { jar.domains().iter().map(|(d, _)| d.to_string()).collect() };

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| AppError::Database(format!("Failed to start transaction: {e}")))?;

    for domain in &all_domains {
        for cookie in jar.cookies_for_domain(domain) {
            tx.execute(
                "INSERT OR REPLACE INTO cookies
                    (domain, name, value, path, expires_at, secure, http_only, same_site, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    cookie.domain,
                    cookie.name,
                    cookie.value,
                    cookie.path,
                    cookie.expires,
                    i32::from(cookie.secure),
                    i32::from(cookie.http_only),
                    cookie.same_site.to_string(),
                    now,
                ],
            )
            .map_err(|e| AppError::Database(format!("Failed to save cookie: {e}")))?;
        }
    }

    tx.commit()
        .map_err(|e| AppError::Database(format!("Failed to commit cookie transaction: {e}")))?;
    Ok(())
}

/// Load all cookies from `SQLite` into a fresh `CookieJar`.
/// Called once at app startup.
pub fn load_cookies(conn: &Connection) -> std::result::Result<crate::cookie::CookieJar, AppError> {
    use crate::cookie::{Cookie, CookieJar, SameSite};

    let mut jar = CookieJar::new();
    let mut stmt = conn
        .prepare(
            "SELECT domain, name, value, path, expires_at, secure, http_only, same_site
             FROM cookies",
        )
        .map_err(|e| AppError::Database(format!("Failed to prepare cookie query: {e}")))?;

    let rows = stmt
        .query_map([], |row| {
            let same_site_str: String = row.get(7)?;
            let same_site = match same_site_str.to_lowercase().as_str() {
                "strict" => SameSite::Strict,
                "none" => SameSite::None,
                _ => SameSite::Lax,
            };
            Ok(Cookie {
                domain: row.get(0)?,
                name: row.get(1)?,
                value: row.get(2)?,
                path: row.get(3)?,
                expires: row.get(4)?,
                secure: row.get::<_, i32>(5)? != 0,
                http_only: row.get::<_, i32>(6)? != 0,
                same_site,
            })
        })
        .map_err(|e| AppError::Database(format!("Failed to query cookies: {e}")))?;

    for row in rows {
        match row {
            Ok(cookie) => jar.insert(cookie),
            Err(e) => log::warn!("Skipping corrupt cookie row: {e}"),
        }
    }

    log::info!(
        "Loaded {} cookies across {} domains from SQLite",
        jar.total_count(),
        jar.domain_count()
    );
    Ok(jar)
}

/// Remove all cookies from the database (used by Clear Cookies button).
pub fn clear_cookies_db(conn: &Connection) -> std::result::Result<(), AppError> {
    conn.execute("DELETE FROM cookies", [])
        .map_err(|e| AppError::Database(format!("Failed to clear cookies: {e}")))?;
    Ok(())
}

/// Remove all cookies for a specific domain.
pub fn clear_domain_cookies_db(
    conn: &Connection,
    domain: &str,
) -> std::result::Result<(), AppError> {
    conn.execute("DELETE FROM cookies WHERE domain = ?1", params![domain])
        .map_err(|e| AppError::Database(format!("Failed to clear domain cookies: {e}")))?;
    Ok(())
}

/// Remove a single cookie by domain, name, and path.
pub fn delete_cookie_db(
    conn: &Connection,
    domain: &str,
    name: &str,
    path: &str,
) -> std::result::Result<(), AppError> {
    conn.execute(
        "DELETE FROM cookies WHERE domain = ?1 AND name = ?2 AND path = ?3",
        params![domain, name, path],
    )
    .map_err(|e| AppError::Database(format!("Failed to delete cookie: {e}")))?;
    Ok(())
}

/// Update a cookie's value (used by inline editor).
pub fn update_cookie_value_db(
    conn: &Connection,
    domain: &str,
    name: &str,
    path: &str,
    new_value: &str,
) -> std::result::Result<(), AppError> {
    conn.execute(
        "UPDATE cookies SET value = ?1 WHERE domain = ?2 AND name = ?3 AND path = ?4",
        params![new_value, domain, name, path],
    )
    .map_err(|e| AppError::Database(format!("Failed to update cookie: {e}")))?;
    Ok(())
}
