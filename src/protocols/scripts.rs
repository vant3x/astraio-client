use crate::error::AppError;
use crate::http_client::request::HttpRequest;
use crate::http_client::response::HttpResponse;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ScriptAction {
    SetVariable {
        name: String,
        value: String,
    },
    SetHeader {
        key: String,
        value: String,
    },
    RemoveHeader {
        key: String,
    },
    SetBody {
        body: String,
    },
    SetUrl {
        url: String,
    },
    SetMethod {
        method: String,
    },
    AssertStatus {
        expected: u16,
    },
    AssertHeader {
        key: String,
        contains: Option<String>,
        equals: Option<String>,
    },
    AssertBody {
        contains: Option<String>,
        equals: Option<String>,
    },
    ExtractJson {
        variable: String,
        path: String,
    },
    ExtractHeader {
        variable: String,
        header: String,
    },
    Log {
        message: String,
    },
    Delay {
        ms: u64,
    },
}

impl std::fmt::Display for ScriptAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScriptAction::SetVariable { name, value } => write!(f, "set_var({}={})", name, value),
            ScriptAction::SetHeader { key, value } => write!(f, "set_header({}: {})", key, value),
            ScriptAction::RemoveHeader { key } => write!(f, "remove_header({})", key),
            ScriptAction::SetBody { body } => {
                let preview: String = body.chars().take(30).collect();
                write!(f, "set_body({}...)", preview)
            }
            ScriptAction::SetUrl { url } => write!(f, "set_url({})", url),
            ScriptAction::SetMethod { method } => write!(f, "set_method({})", method),
            ScriptAction::AssertStatus { expected } => write!(f, "assert_status({})", expected),
            ScriptAction::AssertHeader { key, .. } => write!(f, "assert_header({})", key),
            ScriptAction::AssertBody { .. } => write!(f, "assert_body(...)"),
            ScriptAction::ExtractJson { variable, path } => {
                write!(f, "extract_json({} from {})", variable, path)
            }
            ScriptAction::ExtractHeader { variable, header } => {
                write!(f, "extract_header({} from {})", variable, header)
            }
            ScriptAction::Log { message } => write!(f, "log({})", message),
            ScriptAction::Delay { ms } => write!(f, "delay({}ms)", ms),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct Script {
    pub actions: Vec<ScriptAction>,
}

impl Script {
    pub fn from_json(json: &str) -> Result<Self, AppError> {
        if json.trim().is_empty() {
            return Ok(Self::default());
        }
        serde_json::from_str(json).map_err(|e| AppError::Parse(format!("Invalid script: {}", e)))
    }

    pub fn to_json(&self) -> Result<String, AppError> {
        serde_json::to_string_pretty(self).map_err(|e| AppError::Serialization(e.to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct ScriptContext {
    pub variables: HashMap<String, String>,
    pub logs: Vec<String>,
    pub errors: Vec<String>,
}

impl ScriptContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            logs: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn resolve_variables(&self, input: &str) -> String {
        let mut result = input.to_string();
        for (key, value) in &self.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestScripts {
    #[serde(default)]
    pub pre_request: Script,
    #[serde(default)]
    pub post_response: Script,
}

impl RequestScripts {
    pub fn from_json(json: &str) -> Result<Self, AppError> {
        if json.trim().is_empty() {
            return Ok(Self::default());
        }
        serde_json::from_str(json).map_err(|e| AppError::Parse(format!("Invalid scripts: {}", e)))
    }

    pub fn to_json(&self) -> Result<String, AppError> {
        serde_json::to_string_pretty(self).map_err(|e| AppError::Serialization(e.to_string()))
    }
}

pub struct ScriptEngine;

impl ScriptEngine {
    pub fn execute_pre_request(
        script: &Script,
        request: &mut HttpRequest,
        context: &mut ScriptContext,
    ) -> Result<(), AppError> {
        for action in &script.actions {
            Self::execute_action_pre(action, request, context)?;
        }
        Ok(())
    }

    pub fn execute_post_response(
        script: &Script,
        response: &HttpResponse,
        context: &mut ScriptContext,
    ) -> Result<(), AppError> {
        for action in &script.actions {
            Self::execute_action_post(action, response, context)?;
        }
        Ok(())
    }

    fn execute_action_pre(
        action: &ScriptAction,
        request: &mut HttpRequest,
        context: &mut ScriptContext,
    ) -> Result<(), AppError> {
        match action {
            ScriptAction::SetVariable { name, value } => {
                let resolved = context.resolve_variables(value);
                context.variables.insert(name.clone(), resolved);
            }
            ScriptAction::SetHeader { key, value } => {
                let resolved_key = context.resolve_variables(key);
                let resolved_value = context.resolve_variables(value);
                request
                    .headers
                    .retain(|(k, _)| !k.eq_ignore_ascii_case(&resolved_key));
                request.headers.push((resolved_key, resolved_value));
            }
            ScriptAction::RemoveHeader { key } => {
                let resolved = context.resolve_variables(key);
                request
                    .headers
                    .retain(|(k, _)| !k.eq_ignore_ascii_case(&resolved));
            }
            ScriptAction::SetBody { body } => {
                let resolved = context.resolve_variables(body);
                request.body = Some(resolved);
            }
            ScriptAction::SetUrl { url } => {
                let resolved = context.resolve_variables(url);
                request.url = resolved;
            }
            ScriptAction::SetMethod { method } => {
                let resolved = context.resolve_variables(method);
                let upper = resolved.to_uppercase();
                let valid = matches!(
                    upper.as_str(),
                    "GET"
                        | "POST"
                        | "PUT"
                        | "DELETE"
                        | "PATCH"
                        | "HEAD"
                        | "OPTIONS"
                        | "TRACE"
                        | "CONNECT"
                );
                if !valid {
                    return Err(AppError::Validation(format!(
                        "Invalid HTTP method: {}",
                        resolved
                    )));
                }
                request.method = resolved.parse().map_err(|_| {
                    AppError::Validation(format!("Invalid HTTP method: {}", resolved))
                })?;
            }
            ScriptAction::Log { message } => {
                let resolved = context.resolve_variables(message);
                context.logs.push(resolved);
            }
            ScriptAction::Delay { ms } => {
                log::warn!(
                    "Script Delay({}ms) is not supported in synchronous context - skipping",
                    ms
                );
            }
            _ => {}
        }
        Ok(())
    }

    fn execute_action_post(
        action: &ScriptAction,
        response: &HttpResponse,
        context: &mut ScriptContext,
    ) -> Result<(), AppError> {
        match action {
            ScriptAction::AssertStatus { expected } => {
                if response.status != *expected {
                    let msg = format!(
                        "Assertion failed: expected status {}, got {}",
                        expected, response.status
                    );
                    context.errors.push(msg.clone());
                    return Err(AppError::Validation(msg));
                }
            }
            ScriptAction::AssertHeader {
                key,
                contains,
                equals,
            } => {
                let header_value = response
                    .headers
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(key))
                    .map(|(_, v)| v.as_str());

                match header_value {
                    None => {
                        let msg = format!("Assertion failed: header '{}' not found", key);
                        context.errors.push(msg.clone());
                        return Err(AppError::Validation(msg));
                    }
                    Some(val) => {
                        if let Some(expected) = equals {
                            let resolved = context.resolve_variables(expected);
                            if val != resolved {
                                let msg = format!(
                                    "Assertion failed: header '{}' expected '{}', got '{}'",
                                    key, resolved, val
                                );
                                context.errors.push(msg.clone());
                                return Err(AppError::Validation(msg));
                            }
                        }
                        if let Some(expected) = contains {
                            let resolved = context.resolve_variables(expected);
                            if !val.contains(resolved.as_str()) {
                                let msg = format!(
                                    "Assertion failed: header '{}' should contain '{}', got '{}'",
                                    key, resolved, val
                                );
                                context.errors.push(msg.clone());
                                return Err(AppError::Validation(msg));
                            }
                        }
                    }
                }
            }
            ScriptAction::AssertBody { contains, equals } => {
                if let Some(expected) = equals {
                    let resolved = context.resolve_variables(expected);
                    if response.body != resolved {
                        let msg = format!(
                            "Assertion failed: body expected '{}', got '{}'",
                            resolved,
                            &response.body[..response.body.len().min(100)]
                        );
                        context.errors.push(msg.clone());
                        return Err(AppError::Validation(msg));
                    }
                }
                if let Some(expected) = contains {
                    let resolved = context.resolve_variables(expected);
                    if !response.body.contains(resolved.as_str()) {
                        let msg = format!("Assertion failed: body should contain '{}'", resolved);
                        context.errors.push(msg.clone());
                        return Err(AppError::Validation(msg));
                    }
                }
            }
            ScriptAction::ExtractJson { variable, path } => {
                let resolved_path = context.resolve_variables(path);
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&response.body) {
                    let extracted = extract_json_path(&json_value, &resolved_path);
                    if let Some(val) = extracted {
                        context.variables.insert(variable.clone(), val);
                    } else {
                        context
                            .errors
                            .push(format!("JSON path '{}' not found", resolved_path));
                    }
                }
            }
            ScriptAction::ExtractHeader { variable, header } => {
                let resolved_header = context.resolve_variables(header);
                if let Some((_, value)) = response
                    .headers
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(&resolved_header))
                {
                    context.variables.insert(variable.clone(), value.clone());
                }
            }
            ScriptAction::Log { message } => {
                let resolved = context.resolve_variables(message);
                context.logs.push(resolved);
            }
            ScriptAction::SetVariable { name, value } => {
                let resolved = context.resolve_variables(value);
                context.variables.insert(name.clone(), resolved);
            }
            _ => {}
        }
        Ok(())
    }
}

fn extract_json_path(value: &serde_json::Value, path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for part in &parts {
        if let Ok(idx) = part.parse::<usize>() {
            current = current.get(idx)?;
        } else {
            current = current.get(*part)?;
        }
    }

    match current {
        serde_json::Value::String(s) => Some(s.clone()),
        other => Some(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http_client::config::RequestConfig;
    use crate::http_client::request::HttpMethod;
    use crate::http_client::response::BodyEncoding;
    use std::time::Duration;

    fn make_request() -> HttpRequest {
        HttpRequest {
            method: HttpMethod::Get,
            url: "https://api.example.com/users".to_string(),
            headers: vec![],
            body: None,
            config: RequestConfig::default(),
            multipart_fields: vec![],
            auth: None,
        }
    }

    fn make_response(status: u16, body: &str) -> HttpResponse {
        HttpResponse {
            url: "https://api.example.com/users".to_string(),
            method: HttpMethod::Get,
            status,
            headers: vec![("content-type".to_string(), "application/json".to_string())],
            body: body.to_string(),
            body_encoding: BodyEncoding::Text,
            duration: Duration::from_millis(100),
            size: body.len() as u64,
            redirect_chain: vec![],
        }
    }

    #[test]
    fn script_from_json_empty() {
        let script = Script::from_json("").unwrap();
        assert!(script.actions.is_empty());
    }

    #[test]
    fn script_from_json_valid() {
        let json = r#"[
            {"action": "set_variable", "name": "token", "value": "abc123"},
            {"action": "set_header", "key": "Authorization", "value": "Bearer {{token}}"}
        ]"#;
        let script = Script::from_json(json).unwrap();
        assert_eq!(script.actions.len(), 2);
    }

    #[test]
    fn script_from_json_invalid() {
        let result = Script::from_json("not json");
        assert!(result.is_err());
    }

    #[test]
    fn set_variable_pre_request() {
        let script = Script {
            actions: vec![ScriptAction::SetVariable {
                name: "myvar".to_string(),
                value: "hello".to_string(),
            }],
        };
        let mut request = make_request();
        let mut context = ScriptContext::new();

        ScriptEngine::execute_pre_request(&script, &mut request, &mut context).unwrap();
        assert_eq!(context.variables.get("myvar").unwrap(), "hello");
    }

    #[test]
    fn set_header_pre_request() {
        let script = Script {
            actions: vec![ScriptAction::SetHeader {
                key: "X-Custom".to_string(),
                value: "test-value".to_string(),
            }],
        };
        let mut request = make_request();
        let mut context = ScriptContext::new();

        ScriptEngine::execute_pre_request(&script, &mut request, &mut context).unwrap();
        assert!(request
            .headers
            .iter()
            .any(|(k, v)| k == "X-Custom" && v == "test-value"));
    }

    #[test]
    fn set_header_with_variable() {
        let script = Script {
            actions: vec![
                ScriptAction::SetVariable {
                    name: "token".to_string(),
                    value: "abc123".to_string(),
                },
                ScriptAction::SetHeader {
                    key: "Authorization".to_string(),
                    value: "Bearer {{token}}".to_string(),
                },
            ],
        };
        let mut request = make_request();
        let mut context = ScriptContext::new();

        ScriptEngine::execute_pre_request(&script, &mut request, &mut context).unwrap();
        assert!(request
            .headers
            .iter()
            .any(|(k, v)| k == "Authorization" && v == "Bearer abc123"));
    }

    #[test]
    fn remove_header_pre_request() {
        let script = Script {
            actions: vec![ScriptAction::RemoveHeader {
                key: "X-Remove-Me".to_string(),
            }],
        };
        let mut request = make_request();
        request
            .headers
            .push(("X-Remove-Me".to_string(), "value".to_string()));
        let mut context = ScriptContext::new();

        ScriptEngine::execute_pre_request(&script, &mut request, &mut context).unwrap();
        assert!(!request.headers.iter().any(|(k, _)| k == "X-Remove-Me"));
    }

    #[test]
    fn set_body_pre_request() {
        let script = Script {
            actions: vec![ScriptAction::SetBody {
                body: r#"{"key":"value"}"#.to_string(),
            }],
        };
        let mut request = make_request();
        let mut context = ScriptContext::new();

        ScriptEngine::execute_pre_request(&script, &mut request, &mut context).unwrap();
        assert_eq!(request.body.as_deref(), Some(r#"{"key":"value"}"#));
    }

    #[test]
    fn set_url_pre_request() {
        let script = Script {
            actions: vec![ScriptAction::SetUrl {
                url: "https://other.api.com/data".to_string(),
            }],
        };
        let mut request = make_request();
        let mut context = ScriptContext::new();

        ScriptEngine::execute_pre_request(&script, &mut request, &mut context).unwrap();
        assert_eq!(request.url, "https://other.api.com/data");
    }

    #[test]
    fn set_method_pre_request() {
        let script = Script {
            actions: vec![ScriptAction::SetMethod {
                method: "POST".to_string(),
            }],
        };
        let mut request = make_request();
        let mut context = ScriptContext::new();

        ScriptEngine::execute_pre_request(&script, &mut request, &mut context).unwrap();
        assert_eq!(request.method, HttpMethod::Post);
    }

    #[test]
    fn set_method_invalid_returns_error() {
        let script = Script {
            actions: vec![ScriptAction::SetMethod {
                method: "NOTVALID".to_string(),
            }],
        };
        let mut request = make_request();
        let mut context = ScriptContext::new();

        let result = ScriptEngine::execute_pre_request(&script, &mut request, &mut context);
        assert!(result.is_err());
    }

    #[test]
    fn assert_status_ok() {
        let script = Script {
            actions: vec![ScriptAction::AssertStatus { expected: 200 }],
        };
        let response = make_response(200, "{}");
        let mut context = ScriptContext::new();

        let result = ScriptEngine::execute_post_response(&script, &response, &mut context);
        assert!(result.is_ok());
        assert!(context.errors.is_empty());
    }

    #[test]
    fn assert_status_fail() {
        let script = Script {
            actions: vec![ScriptAction::AssertStatus { expected: 200 }],
        };
        let response = make_response(404, "{}");
        let mut context = ScriptContext::new();

        let result = ScriptEngine::execute_post_response(&script, &response, &mut context);
        assert!(result.is_err());
        assert_eq!(context.errors.len(), 1);
    }

    #[test]
    fn assert_header_contains() {
        let script = Script {
            actions: vec![ScriptAction::AssertHeader {
                key: "content-type".to_string(),
                contains: Some("json".to_string()),
                equals: None,
            }],
        };
        let response = make_response(200, "{}");
        let mut context = ScriptContext::new();

        let result = ScriptEngine::execute_post_response(&script, &response, &mut context);
        assert!(result.is_ok());
    }

    #[test]
    fn assert_header_contains_fail() {
        let script = Script {
            actions: vec![ScriptAction::AssertHeader {
                key: "content-type".to_string(),
                contains: Some("xml".to_string()),
                equals: None,
            }],
        };
        let response = make_response(200, "{}");
        let mut context = ScriptContext::new();

        let result = ScriptEngine::execute_post_response(&script, &response, &mut context);
        assert!(result.is_err());
    }

    #[test]
    fn assert_header_not_found() {
        let script = Script {
            actions: vec![ScriptAction::AssertHeader {
                key: "x-custom".to_string(),
                contains: None,
                equals: Some("value".to_string()),
            }],
        };
        let response = make_response(200, "{}");
        let mut context = ScriptContext::new();

        let result = ScriptEngine::execute_post_response(&script, &response, &mut context);
        assert!(result.is_err());
    }

    #[test]
    fn assert_body_contains() {
        let script = Script {
            actions: vec![ScriptAction::AssertBody {
                contains: Some("users".to_string()),
                equals: None,
            }],
        };
        let response = make_response(200, r#"{"users": []}"#);
        let mut context = ScriptContext::new();

        let result = ScriptEngine::execute_post_response(&script, &response, &mut context);
        assert!(result.is_ok());
    }

    #[test]
    fn assert_body_contains_fail() {
        let script = Script {
            actions: vec![ScriptAction::AssertBody {
                contains: Some("orders".to_string()),
                equals: None,
            }],
        };
        let response = make_response(200, r#"{"users": []}"#);
        let mut context = ScriptContext::new();

        let result = ScriptEngine::execute_post_response(&script, &response, &mut context);
        assert!(result.is_err());
    }

    #[test]
    fn extract_json_field() {
        let script = Script {
            actions: vec![ScriptAction::ExtractJson {
                variable: "user_id".to_string(),
                path: "data.id".to_string(),
            }],
        };
        let response = make_response(200, r#"{"data": {"id": 42, "name": "John"}}"#);
        let mut context = ScriptContext::new();

        ScriptEngine::execute_post_response(&script, &response, &mut context).unwrap();
        assert_eq!(context.variables.get("user_id").unwrap(), "42");
    }

    #[test]
    fn extract_header_value() {
        let script = Script {
            actions: vec![ScriptAction::ExtractHeader {
                variable: "content_type".to_string(),
                header: "content-type".to_string(),
            }],
        };
        let response = make_response(200, "{}");
        let mut context = ScriptContext::new();

        ScriptEngine::execute_post_response(&script, &response, &mut context).unwrap();
        assert_eq!(
            context.variables.get("content_type").unwrap(),
            "application/json"
        );
    }

    #[test]
    fn log_message() {
        let script = Script {
            actions: vec![ScriptAction::Log {
                message: "Request sent".to_string(),
            }],
        };
        let mut request = make_request();
        let mut context = ScriptContext::new();

        ScriptEngine::execute_pre_request(&script, &mut request, &mut context).unwrap();
        assert_eq!(context.logs.len(), 1);
        assert_eq!(context.logs[0], "Request sent");
    }

    #[test]
    fn log_with_variable() {
        let script = Script {
            actions: vec![
                ScriptAction::SetVariable {
                    name: "url".to_string(),
                    value: "https://api.example.com".to_string(),
                },
                ScriptAction::Log {
                    message: "Calling {{url}}".to_string(),
                },
            ],
        };
        let mut request = make_request();
        let mut context = ScriptContext::new();

        ScriptEngine::execute_pre_request(&script, &mut request, &mut context).unwrap();
        assert_eq!(context.logs[0], "Calling https://api.example.com");
    }

    #[test]
    fn context_resolve_variables() {
        let mut context = ScriptContext::new();
        context
            .variables
            .insert("host".to_string(), "api.example.com".to_string());
        context
            .variables
            .insert("version".to_string(), "v2".to_string());

        let resolved = context.resolve_variables("https://{{host}}/{{version}}/users");
        assert_eq!(resolved, "https://api.example.com/v2/users");
    }

    #[test]
    fn request_scripts_json_roundtrip() {
        let scripts = RequestScripts {
            pre_request: Script {
                actions: vec![ScriptAction::SetHeader {
                    key: "X-Request-Time".to_string(),
                    value: "12345".to_string(),
                }],
            },
            post_response: Script {
                actions: vec![ScriptAction::AssertStatus { expected: 200 }],
            },
        };

        let json = scripts.to_json().unwrap();
        let restored = RequestScripts::from_json(&json).unwrap();
        assert_eq!(scripts, restored);
    }

    #[test]
    fn request_scripts_from_empty_json() {
        let scripts = RequestScripts::from_json("").unwrap();
        assert!(scripts.pre_request.actions.is_empty());
        assert!(scripts.post_response.actions.is_empty());
    }

    #[test]
    fn multiple_pre_request_actions() {
        let script = Script {
            actions: vec![
                ScriptAction::SetVariable {
                    name: "token".to_string(),
                    value: "abc123".to_string(),
                },
                ScriptAction::SetHeader {
                    key: "Authorization".to_string(),
                    value: "Bearer {{token}}".to_string(),
                },
                ScriptAction::SetHeader {
                    key: "X-Custom".to_string(),
                    value: "fixed-value".to_string(),
                },
                ScriptAction::SetBody {
                    body: r#"{"token":"{{token}}"}"#.to_string(),
                },
            ],
        };
        let mut request = make_request();
        let mut context = ScriptContext::new();

        ScriptEngine::execute_pre_request(&script, &mut request, &mut context).unwrap();

        assert_eq!(context.variables.get("token").unwrap(), "abc123");
        assert!(request
            .headers
            .iter()
            .any(|(k, v)| k == "Authorization" && v == "Bearer abc123"));
        assert!(request
            .headers
            .iter()
            .any(|(k, v)| k == "X-Custom" && v == "fixed-value"));
        assert_eq!(request.body.as_deref(), Some(r#"{"token":"abc123"}"#));
    }

    #[test]
    fn post_response_chained_extractions() {
        let script = Script {
            actions: vec![
                ScriptAction::ExtractJson {
                    variable: "user_id".to_string(),
                    path: "data.id".to_string(),
                },
                ScriptAction::Log {
                    message: "Extracted user_id={{user_id}}".to_string(),
                },
            ],
        };
        let response = make_response(200, r#"{"data": {"id": 42}}"#);
        let mut context = ScriptContext::new();

        ScriptEngine::execute_post_response(&script, &response, &mut context).unwrap();
        assert_eq!(context.variables.get("user_id").unwrap(), "42");
        assert_eq!(context.logs[0], "Extracted user_id=42");
    }

    #[test]
    fn extract_json_nested_array() {
        let script = Script {
            actions: vec![ScriptAction::ExtractJson {
                variable: "first_name".to_string(),
                path: "users.0.name".to_string(),
            }],
        };
        let response = make_response(200, r#"{"users": [{"name": "Alice"}, {"name": "Bob"}]}"#);
        let mut context = ScriptContext::new();

        ScriptEngine::execute_post_response(&script, &response, &mut context).unwrap();
        assert_eq!(context.variables.get("first_name").unwrap(), "Alice");
    }

    #[test]
    fn extract_json_nonexistent_path() {
        let script = Script {
            actions: vec![ScriptAction::ExtractJson {
                variable: "missing".to_string(),
                path: "data.nonexistent".to_string(),
            }],
        };
        let response = make_response(200, r#"{"data": {"id": 1}}"#);
        let mut context = ScriptContext::new();

        ScriptEngine::execute_post_response(&script, &response, &mut context).unwrap();
        assert!(context.variables.get("missing").is_none());
        assert_eq!(context.errors.len(), 1);
    }

    #[test]
    fn assert_status_on_error_response() {
        let script = Script {
            actions: vec![ScriptAction::AssertStatus { expected: 200 }],
        };
        let response = make_response(500, "Internal Server Error");
        let mut context = ScriptContext::new();

        let result = ScriptEngine::execute_post_response(&script, &response, &mut context);
        assert!(result.is_err());
        assert!(context.errors[0].contains("500"));
    }

    #[test]
    fn script_action_display() {
        assert_eq!(
            ScriptAction::SetVariable {
                name: "x".to_string(),
                value: "1".to_string()
            }
            .to_string(),
            "set_var(x=1)"
        );
        assert_eq!(
            ScriptAction::SetHeader {
                key: "Auth".to_string(),
                value: "Bearer token".to_string()
            }
            .to_string(),
            "set_header(Auth: Bearer token)"
        );
        assert_eq!(
            ScriptAction::RemoveHeader {
                key: "X-Remove".to_string()
            }
            .to_string(),
            "remove_header(X-Remove)"
        );
        assert_eq!(
            ScriptAction::AssertStatus { expected: 200 }.to_string(),
            "assert_status(200)"
        );
        assert_eq!(
            ScriptAction::Log {
                message: "hello".to_string()
            }
            .to_string(),
            "log(hello)"
        );
        assert_eq!(ScriptAction::Delay { ms: 100 }.to_string(), "delay(100ms)");
        assert_eq!(
            ScriptAction::ExtractJson {
                variable: "id".to_string(),
                path: "data.id".to_string()
            }
            .to_string(),
            "extract_json(id from data.id)"
        );
        assert_eq!(
            ScriptAction::ExtractHeader {
                variable: "ct".to_string(),
                header: "content-type".to_string()
            }
            .to_string(),
            "extract_header(ct from content-type)"
        );
    }
}
