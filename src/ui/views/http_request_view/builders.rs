use super::{BodyType, ContentType, HttpRequestView, MultipartEntry};
use crate::data::auth::Auth;
use crate::http_client::request::{MultipartField, MultipartValue};
use crate::persistence::database::Environment;
use iced::widget::text_editor;

impl HttpRequestView {
    pub fn restore_multipart(&mut self, fields: &[MultipartField]) {
        self.multipart_entries = fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                let (value, is_file) = match &field.value {
                    MultipartValue::Text(t) => (t.clone(), false),
                    MultipartValue::File { path, .. } => (path.clone(), true),
                };
                MultipartEntry {
                    id: i,
                    name: field.name.clone(),
                    value,
                    is_file,
                }
            })
            .collect();
        self.multipart_next_id = fields.len();
    }

    pub fn apply_environment(&mut self, env: &Environment) {
        for (key, value) in &env.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            self.url_input = self.url_input.replace(&placeholder, value);

            let new_body = self.body_input.text().replace(&placeholder, value);
            self.body_input = text_editor::Content::with_text(&new_body);

            for entry in &mut self.headers_editor.entries {
                entry.value = entry.value.replace(&placeholder, value);
            }
            for entry in &mut self.params_editor.entries {
                entry.value = entry.value.replace(&placeholder, value);
            }

            match &mut self.auth {
                Auth::BearerToken(token) => {
                    *token = token.replace(&placeholder, value);
                }
                Auth::Basic { user, pass } => {
                    *user = user.replace(&placeholder, value);
                    *pass = pass.replace(&placeholder, value);
                }
                Auth::ApiKey {
                    key, value: val, ..
                } => {
                    *key = key.replace(&placeholder, value);
                    *val = val.replace(&placeholder, value);
                }
                Auth::Digest { user, pass } => {
                    *user = user.replace(&placeholder, value);
                    *pass = pass.replace(&placeholder, value);
                }
                Auth::OAuth2(config) => {
                    config.auth_url = config.auth_url.replace(&placeholder, value);
                    config.token_url = config.token_url.replace(&placeholder, value);
                    config.device_auth_url = config.device_auth_url.replace(&placeholder, value);
                    config.client_id = config.client_id.replace(&placeholder, value);
                    config.client_secret = config.client_secret.replace(&placeholder, value);
                    config.scopes = config.scopes.replace(&placeholder, value);
                    config.redirect_uri = config.redirect_uri.replace(&placeholder, value);
                    config.access_token = config.access_token.replace(&placeholder, value);
                    config.refresh_token = config.refresh_token.replace(&placeholder, value);
                }
                Auth::None => {}
            }
        }
    }

    pub fn build_request(&self) -> crate::http_client::request::HttpRequest {
        let params: Vec<(String, String)> = self
            .params_editor
            .entries
            .iter()
            .filter(|p| !p.key.is_empty())
            .map(|p| (p.key.clone(), p.value.clone()))
            .collect();

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<String>>()
            .join("&");

        let mut final_url = if query_string.is_empty() {
            self.url_input.clone()
        } else if self.url_input.contains('?') {
            format!("{}&{}", self.url_input, query_string)
        } else {
            format!("{}?{}", self.url_input, query_string)
        };

        let mut headers: Vec<(String, String)> = self
            .headers_editor
            .entries
            .iter()
            .filter(|h| !h.key.is_empty())
            .map(|h| (h.key.clone(), h.value.clone()))
            .collect();

        match &self.auth {
            Auth::BearerToken(token) if !token.is_empty() => {
                headers.push(("Authorization".to_string(), format!("Bearer {}", token)));
            }
            Auth::Basic { user, pass } if !user.is_empty() || !pass.is_empty() => {
                use base64::Engine;
                let encoded =
                    base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", user, pass));
                headers.push(("Authorization".to_string(), format!("Basic {}", encoded)));
            }
            Auth::ApiKey {
                key,
                value,
                location,
            } if !key.is_empty() => match location {
                crate::data::auth::ApiKeyLocation::Header => {
                    headers.push((key.clone(), value.clone()));
                }
                crate::data::auth::ApiKeyLocation::Query => {
                    let separator = if final_url.contains('?') { "&" } else { "?" };
                    final_url = format!(
                        "{}{}{}={}",
                        final_url,
                        separator,
                        urlencoding::encode(key),
                        urlencoding::encode(value)
                    );
                }
            },
            Auth::OAuth2(config) if !config.access_token.is_empty() => {
                headers.push((
                    "Authorization".to_string(),
                    format!("Bearer {}", config.access_token),
                ));
            }
            _ => {}
        }

        let body_text = self.body_input.text();
        let body = if body_text.trim().is_empty() {
            None
        } else {
            Some(body_text)
        };

        // Only set Content-Type for text body (multipart sets it automatically)
        if body.is_some() && self.body_type == BodyType::Text {
            let content_type_str = match self.request_content_type {
                ContentType::Json => "application/json",
                ContentType::Text => "text/plain",
                ContentType::Html => "text/html",
                ContentType::Xml => "application/xml",
            };
            headers.push(("Content-Type".to_string(), content_type_str.to_string()));
        }

        // Convert multipart entries to MultipartField
        let multipart_fields: Vec<MultipartField> =
            if self.body_type == BodyType::Multipart {
                self.multipart_entries
                    .iter()
                    .filter(|e| !e.name.is_empty())
                    .map(|e| {
                        if e.is_file {
                            MultipartField {
                                name: e.name.clone(),
                                value: MultipartValue::File {
                                    path: e.value.clone(),
                                    filename: None,
                                },
                            }
                        } else {
                            MultipartField {
                                name: e.name.clone(),
                                value: MultipartValue::Text(e.value.clone()),
                            }
                        }
                    })
                    .collect()
            } else {
                vec![]
            };

        crate::http_client::request::HttpRequest {
            method: self.method.parse().unwrap(),
            url: final_url,
            headers,
            body,
            config: self.request_config.clone(),
            multipart_fields,
            auth: Some(self.auth.clone()),
        }
    }
}
