mod builders;
mod tests;
mod views;

use crate::data::auth::{Auth, AuthType};
use crate::data::auth_input::AuthInput;
use crate::http_client::config::RequestConfig;
use crate::http_client::response::HttpResponse;
use crate::http_client::snippets::SnippetFormat;
use crate::ui::components::key_value_editor::{self, KeyValueEditor};
use crate::ui::request_status::RequestStatus;
use iced::highlighter;
use iced::widget::image::Handle;
use iced::widget::text_editor;
use std::time::Duration;

pub(crate) const LOGO_BG_BYTES: &[u8] = include_bytes!("../../../../assets/astra-bg.png");

pub(crate) static HTTP_METHODS: [&str; 7] =
    ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Json,
    Text,
    Html,
    Xml,
}

impl ContentType {
    pub const ALL: [ContentType; 4] = [
        ContentType::Json,
        ContentType::Text,
        ContentType::Html,
        ContentType::Xml,
    ];
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ContentType::Json => "JSON",
                ContentType::Text => "Text",
                ContentType::Html => "HTML",
                ContentType::Xml => "XML",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BodyType {
    #[default]
    Text,
    Multipart,
}

impl BodyType {
    pub const ALL: [BodyType; 2] = [BodyType::Text, BodyType::Multipart];
}

impl std::fmt::Display for BodyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BodyType::Text => write!(f, "Text"),
            BodyType::Multipart => write!(f, "Multipart/Form-Data"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MultipartEntry {
    pub id: usize,
    pub name: String,
    pub value: String,
    pub is_file: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultipartFieldType {
    Text,
    File,
}

impl MultipartFieldType {
    pub const ALL: [MultipartFieldType; 2] = [MultipartFieldType::Text, MultipartFieldType::File];
}

impl std::fmt::Display for MultipartFieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MultipartFieldType::Text => write!(f, "Text"),
            MultipartFieldType::File => write!(f, "File"),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Message {
    UrlInputChanged(String),
    MethodSelected(String),
    TabSelected(TabId),
    ResponseTabSelected(ResponseTab),
    AuthTypeSelected(AuthType),
    AuthInputChanged(AuthInput),
    HeadersEditor(key_value_editor::Message),
    ParamsEditor(key_value_editor::Message),
    BodyInputChanged(text_editor::Action),
    RequestContentTypeSelected(ContentType),
    SendRequest,
    SetLoading,
    ResponseReceived(Result<HttpResponse, crate::error::AppError>),
    CopyResponse,
    CopyHeaders,
    CopyBody,
    ResponseContentChanged(text_editor::Action),
    CopySelection,
    TimeoutChanged(String),
    FollowRedirectsToggled(bool),
    MaxRedirectsChanged(String),
    BodyTypeSelected(BodyType),
    MultipartNameChanged(usize, String),
    MultipartValueChanged(usize, String),
    MultipartFieldTypeChanged(usize, MultipartFieldType),
    AddMultipartEntry,
    RemoveMultipartEntry(usize),
    MultipartFilePicked(usize, Option<String>),
    MultipartBrowseFile(usize),
    RetryCountChanged(String),
    RetryBackoffChanged(String),
    ProxyUrlChanged(String),
    VerifySslToggled(bool),
    ThemeSelected(highlighter::Theme),
    ShowSnippets,
    HideSnippets,
    SnippetFormatSelected(SnippetFormat),
    CopySnippet,
    ResetSettings,
    ToggleWordWrap,
    OAuth2StartAuth,
    OAuth2RefreshToken,
    OAuth2StartDeviceAuth,
    OAuth2CopyUserCode(String),
    OAuth2CopyAccessToken(String),
    OAuth2CopyRefreshToken(String),
    OAuth2AutoPollToggle(bool),
    CurlImported,
    ToggleResponseSearch,
    ResponseSearchChanged(String),
    SearchNext,
    SearchPrev,
    DownloadResponse,
    ResponseFileSaved(Result<String, String>),
    ToggleImagePreview,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TabId {
    #[default]
    Body,
    Headers,
    Params,
    Authorization,
    Settings,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ResponseTab {
    #[default]
    Body,
    Headers,
    Timeline,
}

pub use crate::ui::theme::method_color;

#[derive(Debug)]
pub struct HttpRequestView {
    pub url_input: String,
    pub method: String,
    pub body_input: text_editor::Content,
    pub auth: Auth,
    pub headers_editor: KeyValueEditor,
    pub params_editor: KeyValueEditor,
    pub(crate) active_tab: TabId,
    pub(crate) active_response_tab: ResponseTab,
    pub(crate) request_status: RequestStatus,
    pub last_response: Option<HttpResponse>,
    pub response_body_editor: text_editor::Content,
    pub status_code: Option<u16>,
    pub content_type: Option<String>,
    pub response_duration: Option<Duration>,
    pub response_size: Option<u64>,
    pub request_content_type: ContentType,
    pub request_config: RequestConfig,
    pub body_type: BodyType,
    pub multipart_entries: Vec<MultipartEntry>,
    pub(crate) multipart_next_id: usize,
    pub highlighter_theme: highlighter::Theme,
    pub show_snippets: bool,
    pub snippet_format: SnippetFormat,
    pub snippet_content: text_editor::Content,
    pub word_wrap: bool,
    pub pending_request_data: Option<String>,
    pub(crate) logo_handle: Handle,
    pub show_response_search: bool,
    pub response_search_query: String,
    pub response_search_matches: Vec<(usize, usize)>,
    pub response_search_index: usize,
    pub show_image_preview: bool,
    pub image_preview_handle: Option<Handle>,
}

impl Clone for HttpRequestView {
    fn clone(&self) -> Self {
        Self {
            url_input: self.url_input.clone(),
            method: self.method.clone(),
            body_input: text_editor::Content::with_text(&self.body_input.text()),
            auth: self.auth.clone(),
            headers_editor: self.headers_editor.clone(),
            params_editor: self.params_editor.clone(),
            active_tab: self.active_tab.clone(),
            active_response_tab: self.active_response_tab.clone(),
            request_status: self.request_status.clone(),
            last_response: self.last_response.clone(),
            response_body_editor: text_editor::Content::with_text(
                &self.response_body_editor.text(),
            ),
            status_code: self.status_code,
            content_type: self.content_type.clone(),
            response_duration: self.response_duration,
            response_size: self.response_size,
            request_content_type: self.request_content_type,
            request_config: self.request_config.clone(),
            body_type: self.body_type,
            multipart_entries: self.multipart_entries.clone(),
            multipart_next_id: self.multipart_next_id,
            highlighter_theme: self.highlighter_theme,
            show_snippets: self.show_snippets,
            snippet_format: self.snippet_format,
            snippet_content: text_editor::Content::with_text(&self.snippet_content.text()),
            word_wrap: self.word_wrap,
            pending_request_data: self.pending_request_data.clone(),
            logo_handle: self.logo_handle.clone(),
            show_response_search: self.show_response_search,
            response_search_query: self.response_search_query.clone(),
            response_search_matches: self.response_search_matches.clone(),
            response_search_index: self.response_search_index,
            show_image_preview: self.show_image_preview,
            image_preview_handle: self.image_preview_handle.clone(),
        }
    }
}

impl Default for HttpRequestView {
    fn default() -> Self {
        Self {
            url_input: "https://jsonplaceholder.typicode.com/todos/1".to_string(),
            method: "GET".to_string(),
            body_input: text_editor::Content::new(),
            auth: Auth::default(),
            headers_editor: KeyValueEditor::new("Add Header".to_string()),
            params_editor: KeyValueEditor::new("Add Param".to_string()),
            active_tab: TabId::Body,
            active_response_tab: ResponseTab::Body,
            request_status: RequestStatus::Idle,
            last_response: None,
            response_body_editor: text_editor::Content::new(),
            status_code: None,
            content_type: None,
            response_duration: None,
            response_size: None,
            request_content_type: ContentType::Json,
            request_config: RequestConfig::default(),
            body_type: BodyType::Text,
            multipart_entries: vec![MultipartEntry {
                id: 0,
                name: String::new(),
                value: String::new(),
                is_file: false,
            }],
            multipart_next_id: 1,
            highlighter_theme: highlighter::Theme::SolarizedDark,
            show_snippets: false,
            snippet_format: SnippetFormat::Curl,
            snippet_content: text_editor::Content::new(),
            word_wrap: false,
            pending_request_data: None,
            logo_handle: Handle::from_bytes(bytes::Bytes::from_static(LOGO_BG_BYTES)),
            show_response_search: false,
            response_search_query: String::new(),
            response_search_matches: Vec::new(),
            response_search_index: 0,
            show_image_preview: false,
            image_preview_handle: None,
        }
    }
}

impl HttpRequestView {
    pub fn is_body_empty(text: &str) -> bool {
        let trimmed = text.trim();
        trimmed.is_empty() || trimmed == "\n"
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::UrlInputChanged(url) => {
                if url.trim_start().starts_with("curl ") {
                    if let Ok(parsed) = crate::import::curl::parse_curl(&url) {
                        self.url_input = parsed.url;
                        self.method = parsed.method;
                        if let Some(body) = parsed.body {
                            self.body_input = text_editor::Content::with_text(&body);
                        }
                        self.headers_editor.entries.clear();
                        for (key, value) in parsed.headers {
                            self.headers_editor.entries.push(
                                crate::ui::components::key_value_editor::KeyValueEntry {
                                    id: self.headers_editor.entries.len(),
                                    key,
                                    value,
                                },
                            );
                        }
                        if let (Some(user), Some(pass)) = (parsed.auth_user, parsed.auth_pass) {
                            self.auth = Auth::Basic { user, pass };
                        }
                        if parsed.insecure {
                            self.request_config.verify_ssl = false;
                        }
                        if !parsed.form_fields.is_empty() {
                            self.body_type = BodyType::Multipart;
                            self.multipart_entries.clear();
                            for (i, (name, value)) in parsed.form_fields.into_iter().enumerate() {
                                let is_file = value.starts_with('@');
                                let file_value = if is_file {
                                    value[1..].to_string()
                                } else {
                                    value
                                };
                                self.multipart_entries.push(MultipartEntry {
                                    id: i,
                                    name,
                                    value: file_value,
                                    is_file,
                                });
                            }
                            self.multipart_next_id = self.multipart_entries.len();
                        }
                    } else {
                        self.url_input = url;
                    }
                } else {
                    self.url_input = url;
                }
            }
            Message::MethodSelected(method) => self.method = method,
            Message::TabSelected(tab_id) => self.active_tab = tab_id,
            Message::ResponseTabSelected(tab_id) => self.active_response_tab = tab_id,
            Message::AuthTypeSelected(auth_type) => {
                self.auth = match auth_type {
                    AuthType::NoAuth => Auth::None,
                    AuthType::BearerToken => Auth::BearerToken(String::new()),
                    AuthType::BasicAuth => Auth::Basic {
                        user: String::new(),
                        pass: String::new(),
                    },
                    AuthType::ApiKey => Auth::ApiKey {
                        key: String::new(),
                        value: String::new(),
                        location: crate::data::auth::ApiKeyLocation::Header,
                    },
                    AuthType::Digest => Auth::Digest {
                        user: String::new(),
                        pass: String::new(),
                    },
                    AuthType::OAuth2 => Auth::OAuth2(Box::default()),
                };
            }
            Message::AuthInputChanged(input) => {
                self.auth.apply_input(input);
            }
            Message::HeadersEditor(msg) => self.headers_editor.update(msg),
            Message::ParamsEditor(msg) => self.params_editor.update(msg),
            Message::BodyInputChanged(action) => self.body_input.perform(action),
            Message::RequestContentTypeSelected(content_type) => {
                self.request_content_type = content_type
            }
            Message::SendRequest => {}
            Message::SetLoading => {
                self.request_status = RequestStatus::Loading;
                self.last_response = None;
                self.response_body_editor = text_editor::Content::new();
                self.status_code = None;
                self.content_type = None;
                self.response_duration = None;
                self.response_size = None;
                self.show_image_preview = false;
                self.image_preview_handle = None;
            }
            Message::ResponseReceived(result) => match result {
                Ok(response) => {
                    self.status_code = Some(response.status);
                    self.response_duration = Some(response.duration);
                    self.response_size = Some(response.size);
                    let content_type = response
                        .headers
                        .iter()
                        .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
                        .map(|(_, v)| v.clone())
                        .unwrap_or_else(|| "unknown".to_string());
                    self.content_type = Some(content_type.clone());

                    let is_image = content_type.contains("image/");
                    let formatted_body = if content_type.contains("application/json") {
                        match serde_json::from_str::<serde_json::Value>(&response.body) {
                            Ok(json_value) => serde_json::to_string_pretty(&json_value)
                                .unwrap_or_else(|_| response.body.clone()),
                            Err(_) => response.body.clone(),
                        }
                    } else if is_image
                        && response.body_encoding
                            == crate::http_client::response::BodyEncoding::Base64
                    {
                        // Decode base64 image and create preview handle
                        if let Ok(bytes) = base64::Engine::decode(
                            &base64::engine::general_purpose::STANDARD,
                            &response.body,
                        ) {
                            self.image_preview_handle =
                                Some(iced::widget::image::Handle::from_bytes(bytes));
                            self.show_image_preview = true;
                            format!(
                                "[Image: {} bytes, base64 decoded for preview]",
                                response.body.len()
                            )
                        } else {
                            response.body.clone()
                        }
                    } else {
                        response.body.clone()
                    };

                    self.response_body_editor = text_editor::Content::with_text(&formatted_body);
                    self.last_response = Some(response);
                    self.request_status = RequestStatus::Success;
                }
                Err(e) => {
                    self.request_status = RequestStatus::Error(format!("Error: {}", e));
                    self.last_response = None;
                    self.response_body_editor = text_editor::Content::new();
                    self.status_code = None;
                    self.content_type = None;
                    self.response_duration = None;
                    self.response_size = None;
                }
            },
            Message::CopyResponse => {
                let text_to_copy = match &self.request_status {
                    RequestStatus::Success => Some(self.response_body_editor.text()),
                    RequestStatus::Error(error_message) => Some(error_message.clone()),
                    _ => None,
                };

                if let Some(text) = text_to_copy {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(text);
                    }
                }
            }
            Message::CopyHeaders => {
                if let Some(response) = &self.last_response {
                    let headers_text = response
                        .headers
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k, v))
                        .collect::<Vec<_>>()
                        .join("\n");
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(headers_text);
                    }
                }
            }
            Message::CopyBody => {
                let text_to_copy = self.response_body_editor.text();
                if !text_to_copy.is_empty() {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(text_to_copy);
                    }
                }
            }
            Message::ResponseContentChanged(action) => {
                self.response_body_editor.perform(action);
            }
            Message::CopySelection => {
                if let Some(selection) = self.response_body_editor.selection() {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(selection);
                    }
                }
            }
            Message::TimeoutChanged(secs) => {
                if let Ok(s) = secs.parse::<u64>() {
                    self.request_config.timeout = std::time::Duration::from_secs(s);
                }
            }
            Message::FollowRedirectsToggled(follow) => {
                use crate::http_client::config::RedirectPolicy;
                self.request_config.redirect_policy = if follow {
                    RedirectPolicy::Follow
                } else {
                    RedirectPolicy::NoFollow
                };
            }
            Message::MaxRedirectsChanged(max) => {
                if let Ok(n) = max.parse::<u32>() {
                    self.request_config.redirect_policy =
                        crate::http_client::config::RedirectPolicy::Limited(n);
                }
            }
            Message::RetryCountChanged(count) => {
                if let Ok(n) = count.parse::<u32>() {
                    self.request_config.retry.max_retries = n;
                }
            }
            Message::RetryBackoffChanged(ms) => {
                if let Ok(n) = ms.parse::<u64>() {
                    self.request_config.retry.backoff_ms = n;
                }
            }
            Message::ProxyUrlChanged(url) => {
                self.request_config.proxy_url = if url.is_empty() { None } else { Some(url) };
            }
            Message::VerifySslToggled(verify) => {
                self.request_config.verify_ssl = verify;
            }
            Message::ThemeSelected(theme) => {
                self.highlighter_theme = theme;
            }
            Message::BodyTypeSelected(body_type) => {
                self.body_type = body_type;
            }
            Message::MultipartNameChanged(id, name) => {
                if let Some(entry) = self.multipart_entries.iter_mut().find(|e| e.id == id) {
                    entry.name = name;
                }
            }
            Message::MultipartValueChanged(id, value) => {
                if let Some(entry) = self.multipart_entries.iter_mut().find(|e| e.id == id) {
                    entry.value = value;
                }
            }
            Message::MultipartFieldTypeChanged(id, field_type) => {
                if let Some(entry) = self.multipart_entries.iter_mut().find(|e| e.id == id) {
                    entry.is_file = matches!(field_type, MultipartFieldType::File);
                    if !entry.is_file {
                        entry.value.clear();
                    }
                }
            }
            Message::AddMultipartEntry => {
                self.multipart_entries.push(MultipartEntry {
                    id: self.multipart_next_id,
                    name: String::new(),
                    value: String::new(),
                    is_file: false,
                });
                self.multipart_next_id += 1;
            }
            Message::RemoveMultipartEntry(id) => {
                self.multipart_entries.retain(|e| e.id != id);
            }
            Message::ShowSnippets => {
                self.show_snippets = true;
                let request = self.build_request();
                let code = crate::http_client::snippets::generate(&request, self.snippet_format);
                self.snippet_content = text_editor::Content::with_text(&code);
            }
            Message::HideSnippets => {
                self.show_snippets = false;
            }
            Message::SnippetFormatSelected(format) => {
                self.snippet_format = format;
                let request = self.build_request();
                let code = crate::http_client::snippets::generate(&request, self.snippet_format);
                self.snippet_content = text_editor::Content::with_text(&code);
            }
            Message::CopySnippet => {
                let text = self.snippet_content.text();
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(text);
                }
            }
            Message::MultipartBrowseFile(_) => {
                // Handled in app.rs
            }
            Message::MultipartFilePicked(id, path) => {
                if let Some(value) = path {
                    if let Some(entry) = self.multipart_entries.iter_mut().find(|e| e.id == id) {
                        entry.value = value;
                    }
                }
            }
            Message::ResetSettings => {
                self.request_config = RequestConfig::default();
            }
            Message::ToggleWordWrap => {
                self.word_wrap = !self.word_wrap;
            }
            Message::OAuth2StartAuth => {
                // Handled in app.rs
            }
            Message::OAuth2RefreshToken => {
                // Handled in app.rs
            }
            Message::OAuth2StartDeviceAuth => {
                // Handled in app.rs
            }
            Message::OAuth2CopyUserCode(code) => {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(code);
                }
            }
            Message::OAuth2CopyAccessToken(token) => {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(token);
                }
            }
            Message::OAuth2CopyRefreshToken(token) => {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(token);
                }
            }
            Message::OAuth2AutoPollToggle(_) => {
                // Handled in app.rs
            }
            Message::CurlImported => {
                // Handled in app.rs to show toast
            }
            Message::ToggleResponseSearch => {
                self.show_response_search = !self.show_response_search;
                if !self.show_response_search {
                    self.response_search_query.clear();
                    self.response_search_matches.clear();
                    self.response_search_index = 0;
                }
            }
            Message::ResponseSearchChanged(query) => {
                self.response_search_query = query;
                self.update_search_matches();
            }
            Message::SearchNext => {
                if !self.response_search_matches.is_empty() {
                    self.response_search_index =
                        (self.response_search_index + 1) % self.response_search_matches.len();
                }
            }
            Message::SearchPrev => {
                if !self.response_search_matches.is_empty() {
                    self.response_search_index = if self.response_search_index == 0 {
                        self.response_search_matches.len() - 1
                    } else {
                        self.response_search_index - 1
                    };
                }
            }
            Message::DownloadResponse => {
                // Handled in app.rs to use async file dialog
            }
            Message::ResponseFileSaved(_result) => {
                // Toast is handled in app.rs
            }
            Message::ToggleImagePreview => {
                self.show_image_preview = !self.show_image_preview;
            }
        }
    }

    fn update_search_matches(&mut self) {
        self.response_search_matches.clear();
        self.response_search_index = 0;
        if self.response_search_query.is_empty() {
            return;
        }
        let body_text = self.response_body_editor.text();
        let query_lower = self.response_search_query.to_lowercase();
        let body_lower = body_text.to_lowercase();
        let mut start = 0;
        while let Some(pos) = body_lower[start..].find(&query_lower) {
            let absolute_pos = start + pos;
            let line = body_text[..absolute_pos].lines().count();
            let col = absolute_pos
                - body_text[..absolute_pos]
                    .rfind('\n')
                    .map(|p| p + 1)
                    .unwrap_or(0);
            self.response_search_matches.push((line, col));
            start = absolute_pos + 1;
        }
    }
}
