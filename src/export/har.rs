use crate::http_client::request::HttpRequest;
use crate::http_client::response::HttpResponse;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct HarLog {
    pub version: String,
    pub creator: HarCreator,
    pub entries: Vec<HarEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarCreator {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarEntry {
    #[serde(rename = "startedDateTime")]
    pub started_iso: String,
    pub time: u64,
    pub request: HarRequest,
    pub response: HarResponse,
    pub timings: HarTimings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarRequest {
    pub method: String,
    pub url: String,
    #[serde(rename = "httpVersion")]
    pub http_version: String,
    pub cookies: Vec<serde_json::Value>,
    pub headers: Vec<HarNameValue>,
    #[serde(rename = "queryString")]
    pub query_string: Vec<HarNameValuePair>,
    #[serde(rename = "postData")]
    pub post_data: Option<HarPostData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarResponse {
    pub status: u16,
    #[serde(rename = "statusText")]
    pub status_text: String,
    #[serde(rename = "httpVersion")]
    pub http_version: String,
    pub cookies: Vec<serde_json::Value>,
    pub headers: Vec<HarNameValue>,
    pub content: HarContent,
    #[serde(rename = "redirectUrl")]
    pub redirect_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarContent {
    pub size: u64,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarTimings {
    pub send: u64,
    pub wait: u64,
    pub receive: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarNameValue {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarNameValuePair {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HarPostData {
    pub mime_type: String,
    pub text: Option<String>,
}

fn now_iso() -> String {
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Simple ISO 8601 without external dependency
    format!("{}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        1970 + (secs / 31536000) as u32,
        ((secs % 31536000) / 2592000) % 12 + 1,
        ((secs % 2592000) / 86400) + 1,
        (secs % 86400) / 3600,
        (secs % 3600) / 60,
        secs % 60,
    )
}

pub fn export_entries(entries: &[(&HttpRequest, &HttpResponse)]) -> String {
    let har_entries: Vec<HarEntry> = entries
        .iter()
        .map(|(req, resp)| {
            let url = req.url.clone();
            let parsed_url = reqwest::Url::parse(&url);

            let query_string: Vec<HarNameValuePair> = parsed_url
                .as_ref()
                .ok()
                .map(|u| {
                    u.query_pairs()
                        .map(|(k, v)| HarNameValuePair {
                            name: k.to_string(),
                            value: v.to_string(),
                        })
                        .collect()
                })
                .unwrap_or_default();

            let headers: Vec<HarNameValue> = req
                .headers
                .iter()
                .map(|(k, v)| HarNameValue {
                    name: k.clone(),
                    value: v.clone(),
                })
                .collect();

            let resp_headers: Vec<HarNameValue> = resp
                .headers
                .iter()
                .map(|(k, v)| HarNameValue {
                    name: k.clone(),
                    value: v.clone(),
                })
                .collect();

            let content_type = resp
                .headers
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| "text/plain".to_string());

            let post_data = req.body.as_ref().map(|body| HarPostData {
                mime_type: req
                    .headers
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
                    .map(|(_, v)| v.clone())
                    .unwrap_or_else(|| "application/json".to_string()),
                text: Some(body.clone()),
            });

            let duration_ms = resp.duration.as_millis() as u64;

            HarEntry {
                started_iso: now_iso(),
                time: duration_ms,
                request: HarRequest {
                    method: req.method.to_string(),
                    url: url.clone(),
                    http_version: "HTTP/1.1".to_string(),
                    cookies: vec![],
                    headers,
                    query_string,
                    post_data,
                },
                response: HarResponse {
                    status: resp.status,
                    status_text: status_text(resp.status),
                    http_version: "HTTP/1.1".to_string(),
                    cookies: vec![],
                    headers: resp_headers,
                    content: HarContent {
                        size: resp.size,
                        mime_type: content_type,
                        text: if resp.body_encoding == crate::http_client::response::BodyEncoding::Text {
                            Some(resp.body.clone())
                        } else {
                            None
                        },
                    },
                    redirect_url: String::new(),
                },
                timings: HarTimings {
                    send: 0,
                    wait: duration_ms,
                    receive: 0,
                },
            }
        })
        .collect();

    let har = HarLog {
        version: "1.2".to_string(),
        creator: HarCreator {
            name: "AstraNova Client".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        entries: har_entries,
    };

    // HAR spec wraps everything in a "log" key
    let wrapper = serde_json::json!({ "log": har });
    serde_json::to_string_pretty(&wrapper).unwrap_or_else(|_| "{}".to_string())
}

fn status_text(status: u16) -> String {
    match status {
        200 => "OK".to_string(),
        201 => "Created".to_string(),
        204 => "No Content".to_string(),
        301 => "Moved Permanently".to_string(),
        302 => "Found".to_string(),
        304 => "Not Modified".to_string(),
        400 => "Bad Request".to_string(),
        401 => "Unauthorized".to_string(),
        403 => "Forbidden".to_string(),
        404 => "Not Found".to_string(),
        500 => "Internal Server Error".to_string(),
        502 => "Bad Gateway".to_string(),
        503 => "Service Unavailable".to_string(),
        _ => format!("Status {}", status),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http_client::config::RequestConfig;
    use crate::http_client::request::HttpMethod;
    use crate::http_client::response::BodyEncoding;
    use std::time::Duration;

    #[test]
    fn export_single_entry() {
        let req = HttpRequest {
            method: HttpMethod::Get,
            url: "https://api.example.com/users?page=1".to_string(),
            headers: vec![
                ("Accept".to_string(), "application/json".to_string()),
                ("Authorization".to_string(), "Bearer token".to_string()),
            ],
            body: None,
            config: RequestConfig::default(),
            multipart_fields: vec![],
            auth: None,
        };
        let resp = HttpResponse {
            url: "https://api.example.com/users?page=1".to_string(),
            method: HttpMethod::Get,
            status: 200,
            headers: vec![
                ("content-type".to_string(), "application/json".to_string()),
                ("x-request-id".to_string(), "abc123".to_string()),
            ],
            body: r#"{"users": []}"#.to_string(),
            body_encoding: BodyEncoding::Text,
            duration: Duration::from_millis(150),
            size: 16,
            redirect_chain: vec![],
        };

        let har = export_entries(&[(&req, &resp)]);
        let parsed: serde_json::Value = serde_json::from_str(&har).unwrap();

        assert_eq!(parsed["log"]["version"], "1.2");
        assert_eq!(parsed["log"]["creator"]["name"], "AstraNova Client");
        assert_eq!(parsed["log"]["entries"].as_array().unwrap().len(), 1);

        let entry = &parsed["log"]["entries"][0];
        assert_eq!(entry["request"]["method"], "GET");
        assert_eq!(entry["request"]["url"], "https://api.example.com/users?page=1");
        assert_eq!(entry["response"]["status"], 200);
        assert_eq!(entry["response"]["content"]["text"], r#"{"users": []}"#);
        assert_eq!(entry["time"], 150);
    }

    #[test]
    fn export_post_with_body() {
        let req = HttpRequest {
            method: HttpMethod::Post,
            url: "https://api.example.com/users".to_string(),
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
            ],
            body: Some(r#"{"name": "John"}"#.to_string()),
            config: RequestConfig::default(),
            multipart_fields: vec![],
            auth: None,
        };
        let resp = HttpResponse {
            url: "https://api.example.com/users".to_string(),
            method: HttpMethod::Post,
            status: 201,
            headers: vec![],
            body: r#"{"id": 1}"#.to_string(),
            body_encoding: BodyEncoding::Text,
            duration: Duration::from_millis(200),
            size: 11,
            redirect_chain: vec![],
        };

        let har = export_entries(&[(&req, &resp)]);
        let parsed: serde_json::Value = serde_json::from_str(&har).unwrap();
        let entry = &parsed["log"]["entries"][0];

        assert_eq!(entry["request"]["method"], "POST");
        assert_eq!(entry["request"]["postData"]["text"],
            r#"{"name": "John"}"#
        );
        assert_eq!(entry["response"]["status"], 201);
    }

    #[test]
    fn export_multiple_entries() {
        let req1 = HttpRequest {
            method: HttpMethod::Get,
            url: "https://api.example.com/a".to_string(),
            headers: vec![],
            body: None,
            config: RequestConfig::default(),
            multipart_fields: vec![],
            auth: None,
        };
        let resp1 = HttpResponse {
            url: "https://api.example.com/a".to_string(),
            method: HttpMethod::Get,
            status: 200,
            headers: vec![],
            body: "ok".to_string(),
            body_encoding: BodyEncoding::Text,
            duration: Duration::from_millis(50),
            size: 2,
            redirect_chain: vec![],
        };
        let req2 = HttpRequest {
            method: HttpMethod::Delete,
            url: "https://api.example.com/b".to_string(),
            headers: vec![],
            body: None,
            config: RequestConfig::default(),
            multipart_fields: vec![],
            auth: None,
        };
        let resp2 = HttpResponse {
            url: "https://api.example.com/b".to_string(),
            method: HttpMethod::Delete,
            status: 204,
            headers: vec![],
            body: String::new(),
            body_encoding: BodyEncoding::Text,
            duration: Duration::from_millis(30),
            size: 0,
            redirect_chain: vec![],
        };

        let har = export_entries(&[(&req1, &resp1), (&req2, &resp2)]);
        let parsed: serde_json::Value = serde_json::from_str(&har).unwrap();
        assert_eq!(parsed["log"]["entries"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn status_text_map() {
        assert_eq!(status_text(200), "OK");
        assert_eq!(status_text(404), "Not Found");
        assert_eq!(status_text(500), "Internal Server Error");
        assert_eq!(status_text(999), "Status 999");
    }

    #[test]
    fn export_includes_query_params() {
        let req = HttpRequest {
            method: HttpMethod::Get,
            url: "https://api.example.com/search?q=rust&limit=10".to_string(),
            headers: vec![],
            body: None,
            config: RequestConfig::default(),
            multipart_fields: vec![],
            auth: None,
        };
        let resp = HttpResponse {
            url: "https://api.example.com/search?q=rust&limit=10".to_string(),
            method: HttpMethod::Get,
            status: 200,
            headers: vec![],
            body: "[]".to_string(),
            body_encoding: BodyEncoding::Text,
            duration: Duration::from_millis(100),
            size: 2,
            redirect_chain: vec![],
        };

        let har = export_entries(&[(&req, &resp)]);
        let parsed: serde_json::Value = serde_json::from_str(&har).unwrap();
        let qs = &parsed["log"]["entries"][0]["request"]["queryString"];
        assert_eq!(qs.as_array().unwrap().len(), 2);
    }
}
