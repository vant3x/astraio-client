use std::time::SystemTime;

pub fn timestamp_seconds() -> String {
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    duration.as_secs().to_string()
}

#[allow(dead_code)]
pub fn timestamp_millis() -> u64 {
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    duration.as_millis() as u64
}

/// Merge collection variables with environment variables.
/// Collection variables take precedence over environment variables (like Postman).
pub fn merge_variables(
    collection_vars: &[(String, String)],
    env_vars: &[(String, String)],
) -> Vec<(String, String)> {
    let mut merged: Vec<(String, String)> = env_vars.to_vec();
    for (key, value) in collection_vars {
        if let Some(existing) = merged.iter_mut().find(|(k, _)| k == key) {
            *existing = (key.clone(), value.clone());
        } else {
            merged.push((key.clone(), value.clone()));
        }
    }
    merged
}
