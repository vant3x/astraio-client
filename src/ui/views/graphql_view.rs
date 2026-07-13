use crate::data::auth::Auth;
use crate::data::auth::AuthType;
use crate::data::auth_input::AuthInput;
use crate::http_client::config::RequestConfig;
use crate::protocols::graphql::{GraphQLRequest, GraphQLResponse};
use crate::protocols::graphql_schema::{get_autocomplete_suggestions, GraphQLSchema, SchemaType, TypeKind};
use crate::ui::components::key_value_editor::{self, KeyValueEditor};
use crate::ui::request_status::RequestStatus;
use crate::ui::theme::{method_color, status_color};
use base64::{engine::general_purpose, Engine as _};
use iced::highlighter;
use iced::widget::text_editor;
use iced::{
    widget::{button, column, container, row, rule, scrollable, text, text_input},
    Alignment, Color, Element, Length, Renderer, Theme,
};
use iced_aw::{ContextMenu, TabLabel, Tabs};
use iced_fonts::lucide;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabId {
    #[default]
    Query,
    Variables,
    Headers,
    Authorization,
    Schema,
}

impl std::fmt::Display for TabId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TabId::Query => write!(f, "Query"),
            TabId::Variables => write!(f, "Variables"),
            TabId::Headers => write!(f, "Headers"),
            TabId::Authorization => write!(f, "Authorization"),
            TabId::Schema => write!(f, "Schema"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResponseTab {
    #[default]
    Body,
    Headers,
}

#[derive(Debug, Clone)]
pub enum Message {
    UrlInputChanged(String),
    QueryChanged(text_editor::Action),
    VariablesChanged(text_editor::Action),
    OperationNameChanged(String),
    TabSelected(TabId),
    ResponseTabSelected(ResponseTab),
    HeadersEditor(key_value_editor::Message),
    AuthTypeSelected(AuthType),
    AuthInputChanged(AuthInput),
    SendRequest,
    SetLoading,
    ResponseReceived(
        #[allow(clippy::type_complexity)]
        Result<
            (
                GraphQLResponse,
                u16,
                Vec<(String, String)>,
                std::time::Duration,
                u64,
            ),
            crate::error::AppError,
        >,
    ),
    CopyResponse,
    #[allow(dead_code)]
    CopyHeaders,
    CopyBody,
    CopySelection,
    ResponseContentChanged(text_editor::Action),
    ToggleWordWrap,
    ValidateQuery,
    #[allow(dead_code)]
    QueryValidated(Result<(), crate::error::AppError>),
    IntrospectSchema,
    SchemaReceived(Result<GraphQLSchema, crate::error::AppError>),
    SchemaSearchChanged(String),
    SchemaTypeSelected(String),
    SaveToHistory,
    SavedToHistory(Result<(), crate::error::AppError>),
    SaveToCollection(i32, Option<i32>),
    SavedToCollection(Result<(), crate::error::AppError>),
    ToggleSaveMenu,
    AutocompleteSelected(String),
}

#[derive(Debug)]
pub struct GraphQLView {
    pub url_input: String,
    pub query_input: text_editor::Content,
    pub variables_input: text_editor::Content,
    pub operation_name: String,
    pub headers_editor: KeyValueEditor,
    pub auth: Auth,
    pub request_config: RequestConfig,
    active_tab: TabId,
    active_response_tab: ResponseTab,
    request_status: RequestStatus,
    pub last_response: Option<GraphQLResponse>,
    pub response_body_editor: text_editor::Content,
    pub status_code: Option<u16>,
    pub content_type: Option<String>,
    pub response_duration: Option<std::time::Duration>,
    pub response_size: Option<u64>,
    pub highlighter_theme: highlighter::Theme,
    pub word_wrap: bool,
    pub query_validation: Option<Result<(), crate::error::AppError>>,
    pub schema: Option<GraphQLSchema>,
    pub schema_loading: bool,
    pub schema_search: String,
    pub schema_selected_type: Option<String>,
    pub show_save_menu: bool,
    pub last_save_status: Option<String>,
    pub autocomplete_suggestions: Vec<String>,
}

impl Clone for GraphQLView {
    fn clone(&self) -> Self {
        Self {
            url_input: self.url_input.clone(),
            query_input: text_editor::Content::with_text(&self.query_input.text()),
            variables_input: text_editor::Content::with_text(&self.variables_input.text()),
            operation_name: self.operation_name.clone(),
            headers_editor: self.headers_editor.clone(),
            auth: self.auth.clone(),
            request_config: self.request_config.clone(),
            active_tab: self.active_tab,
            active_response_tab: self.active_response_tab,
            request_status: self.request_status.clone(),
            last_response: self.last_response.clone(),
            response_body_editor: text_editor::Content::with_text(
                &self.response_body_editor.text(),
            ),
            status_code: self.status_code,
            content_type: self.content_type.clone(),
            response_duration: self.response_duration,
            response_size: self.response_size,
            highlighter_theme: self.highlighter_theme,
            word_wrap: self.word_wrap,
            query_validation: self.query_validation.clone(),
            schema: self.schema.clone(),
            schema_loading: self.schema_loading,
            schema_search: self.schema_search.clone(),
            schema_selected_type: self.schema_selected_type.clone(),
            show_save_menu: self.show_save_menu,
            last_save_status: self.last_save_status.clone(),
            autocomplete_suggestions: self.autocomplete_suggestions.clone(),
        }
    }
}

impl Default for GraphQLView {
    fn default() -> Self {
        Self {
            url_input: "https://countries.trevorblades.com/".to_string(),
            query_input: text_editor::Content::with_text(
                r#"{
  countries {
    code
    name
    emoji
  }
}"#,
            ),
            variables_input: text_editor::Content::new(),
            operation_name: String::new(),
            headers_editor: KeyValueEditor::new("Add Header".to_string()),
            auth: Auth::default(),
            request_config: RequestConfig::default(),
            active_tab: TabId::Query,
            active_response_tab: ResponseTab::Body,
            request_status: RequestStatus::Idle,
            last_response: None,
            response_body_editor: text_editor::Content::new(),
            status_code: None,
            content_type: None,
            response_duration: None,
            response_size: None,
            highlighter_theme: highlighter::Theme::SolarizedDark,
            word_wrap: false,
            query_validation: None,
            schema: None,
            schema_loading: false,
            schema_search: String::new(),
            schema_selected_type: None,
            show_save_menu: false,
            last_save_status: None,
            autocomplete_suggestions: Vec::new(),
        }
    }
}

impl GraphQLView {
    pub fn apply_environment(&mut self, env: &crate::persistence::database::Environment) {
        for (key, value) in &env.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            self.url_input = self.url_input.replace(&placeholder, value);

            let new_query = self.query_input.text().replace(&placeholder, value);
            self.query_input = text_editor::Content::with_text(&new_query);

            let new_vars = self.variables_input.text().replace(&placeholder, value);
            self.variables_input = text_editor::Content::with_text(&new_vars);

            self.operation_name = self.operation_name.replace(&placeholder, value);

            for entry in &mut self.headers_editor.entries {
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

    fn update_autocomplete(&mut self) {
        if let Some(schema) = &self.schema {
            let query = self.query_input.text();
            let cursor = query.len();
            self.autocomplete_suggestions =
                get_autocomplete_suggestions(schema, &query, cursor);
        }
    }

    pub fn build_request(&self) -> Result<GraphQLRequest, crate::error::AppError> {
        let query = self.query_input.text();
        crate::protocols::graphql::validate_query(&query)?;

        let variables = if self.variables_input.text().trim().is_empty() {
            None
        } else {
            Some(crate::protocols::graphql::parse_variables(
                &self.variables_input.text(),
            )?)
        };

        let operation_name = if self.operation_name.trim().is_empty() {
            None
        } else {
            Some(self.operation_name.clone())
        };

        Ok(GraphQLRequest {
            query,
            variables,
            operation_name,
        })
    }

    pub fn build_http_request(&self) -> crate::http_client::request::HttpRequest {
        let graphql_request = self.build_request().unwrap_or_else(|_| GraphQLRequest {
            query: String::new(),
            variables: None,
            operation_name: None,
        });

        let mut headers: Vec<(String, String)> = self
            .headers_editor
            .entries
            .iter()
            .filter(|h| !h.key.is_empty())
            .map(|h| (h.key.clone(), h.value.clone()))
            .collect();

        headers.push(("Content-Type".to_string(), "application/json".to_string()));

        let mut final_url = self.url_input.clone();

        match &self.auth {
            Auth::BearerToken(token) if !token.is_empty() => {
                headers.push(("Authorization".to_string(), format!("Bearer {}", token)));
            }
            Auth::Basic { user, pass } if !user.is_empty() || !pass.is_empty() => {
                let encoded = general_purpose::STANDARD.encode(format!("{}:{}", user, pass));
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
            _ => {}
        }

        let body = graphql_request.to_json().unwrap_or_default();

        crate::http_client::request::HttpRequest {
            method: crate::http_client::request::HttpMethod::Post,
            url: final_url,
            headers,
            body: Some(body),
            config: self.request_config.clone(),
            multipart_fields: vec![],
            auth: Some(self.auth.clone()),
        }
    }

    pub fn build_introspection_request(&self) -> crate::http_client::request::HttpRequest {
        let graphql_request = GraphQLRequest::new(crate::protocols::graphql_schema::INTROSPECTION_QUERY);

        let mut headers: Vec<(String, String)> = self
            .headers_editor
            .entries
            .iter()
            .filter(|h| !h.key.is_empty())
            .map(|h| (h.key.clone(), h.value.clone()))
            .collect();

        headers.push(("Content-Type".to_string(), "application/json".to_string()));

        let mut final_url = self.url_input.clone();

        match &self.auth {
            Auth::BearerToken(token) if !token.is_empty() => {
                headers.push(("Authorization".to_string(), format!("Bearer {}", token)));
            }
            Auth::Basic { user, pass } if !user.is_empty() || !pass.is_empty() => {
                let encoded = general_purpose::STANDARD.encode(format!("{}:{}", user, pass));
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
            _ => {}
        }

        let body = graphql_request.to_json().unwrap_or_default();

        crate::http_client::request::HttpRequest {
            method: crate::http_client::request::HttpMethod::Post,
            url: final_url,
            headers,
            body: Some(body),
            config: self.request_config.clone(),
            multipart_fields: vec![],
            auth: Some(self.auth.clone()),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::UrlInputChanged(url) => self.url_input = url,
            Message::QueryChanged(action) => {
                self.query_input.perform(action);
                self.update_autocomplete();
            }
            Message::VariablesChanged(action) => self.variables_input.perform(action),
            Message::OperationNameChanged(name) => self.operation_name = name,
            Message::TabSelected(tab) => self.active_tab = tab,
            Message::ResponseTabSelected(tab) => self.active_response_tab = tab,
            Message::HeadersEditor(msg) => self.headers_editor.update(msg),
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
            Message::SendRequest => {}
            Message::SetLoading => {
                self.request_status = RequestStatus::Loading;
                self.last_response = None;
                self.response_body_editor = text_editor::Content::new();
                self.status_code = None;
                self.content_type = None;
                self.response_duration = None;
                self.response_size = None;
            }
            Message::ResponseReceived(result) => match result {
                Ok((response, status, headers, duration, size)) => {
                    self.status_code = Some(status);
                    self.response_duration = Some(duration);
                    self.response_size = Some(size);
                    let ct = headers
                        .iter()
                        .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
                        .map(|(_, v)| v.clone())
                        .unwrap_or_else(|| "application/json".to_string());
                    self.content_type = Some(ct);

                    let formatted = crate::protocols::graphql::format_response(&response);
                    self.response_body_editor = text_editor::Content::with_text(&formatted);
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
                let text = self.response_body_editor.text();
                if !text.is_empty() {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(text);
                    }
                }
            }
            Message::CopyHeaders => {
                if let Some(response) = &self.last_response {
                    if let Ok(json) = serde_json::to_string_pretty(response) {
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            let _ = clipboard.set_text(&json);
                        }
                    }
                }
            }
            Message::CopyBody => {
                let text = self.response_body_editor.text();
                if !text.is_empty() {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(text);
                    }
                }
            }
            Message::CopySelection => {
                if let Some(selection) = self.response_body_editor.selection() {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(selection);
                    }
                }
            }
            Message::ResponseContentChanged(action) => {
                self.response_body_editor.perform(action);
            }
            Message::ToggleWordWrap => {
                self.word_wrap = !self.word_wrap;
            }
            Message::ValidateQuery => {
                self.query_validation = Some(crate::protocols::graphql::validate_query(
                    &self.query_input.text(),
                ));
            }
            Message::QueryValidated(result) => {
                self.query_validation = Some(result);
            }
            Message::IntrospectSchema => {
                self.schema_loading = true;
            }
            Message::SchemaReceived(result) => {
                self.schema_loading = false;
                match result {
                    Ok(schema) => {
                        self.schema = Some(schema);
                    }
                    Err(e) => {
                        self.last_save_status = Some(format!("Schema fetch failed: {}", e));
                    }
                }
            }
            Message::SchemaSearchChanged(search) => {
                self.schema_search = search;
            }
            Message::SchemaTypeSelected(type_name) => {
                self.schema_selected_type = Some(type_name);
            }
            Message::SaveToHistory => {
                self.show_save_menu = false;
            }
            Message::SavedToHistory(result) => {
                match result {
                    Ok(()) => {
                        self.last_save_status = Some("Saved to history".to_string());
                    }
                    Err(e) => {
                        self.last_save_status = Some(format!("Failed to save: {}", e));
                    }
                }
            }
            Message::SaveToCollection(_, _) => {
                self.show_save_menu = false;
            }
            Message::SavedToCollection(result) => {
                match result {
                    Ok(()) => {
                        self.last_save_status = Some("Saved to collection".to_string());
                    }
                    Err(e) => {
                        self.last_save_status = Some(format!("Failed to save: {}", e));
                    }
                }
            }
            Message::ToggleSaveMenu => {
                self.show_save_menu = !self.show_save_menu;
            }
            Message::AutocompleteSelected(suggestion) => {
                let query = self.query_input.text();
                let last_word_start = query
                    .rfind(|c: char| c.is_whitespace() || c == '{' || c == '(' || c == ':')
                    .map(|p| p + 1)
                    .unwrap_or(0);
                let prefix = &query[..last_word_start];
                let new_query = format!("{}{}", prefix, suggestion);
                self.query_input = text_editor::Content::with_text(&new_query);
                self.autocomplete_suggestions.clear();
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message, Theme, Renderer> {
        let url_bar = row![
            text("POST").size(14).color(method_color("POST")),
            text_input("GraphQL endpoint URL", &self.url_input)
                .on_input(Message::UrlInputChanged)
                .padding(10),
            button(row![lucide::send().size(14), text(" Send")].spacing(4))
                .on_press(Message::SendRequest),
            button(row![lucide::database().size(14), text(" Introspect")].spacing(4))
                .on_press(Message::IntrospectSchema),
            {
                let save_button: Element<'_, Message, Theme, Renderer> = button(
                    row![lucide::save().size(14), text(" Save")].spacing(4),
                )
                .on_press(Message::ToggleSaveMenu)
                .into();
                if self.show_save_menu {
                    let menu: Element<'_, Message, Theme, Renderer> = column![
                        button(text("Save to History").size(12))
                            .on_press(Message::SaveToHistory),
                        button(text("Save to Collection").size(12))
                            .on_press(Message::SaveToCollection(0, None)),
                    ]
                    .padding(5)
                    .spacing(2)
                    .into();
                    container(column![save_button, menu]).into()
                } else {
                    save_button
                }
            },
        ]
        .spacing(10)
        .padding(10)
        .align_y(Alignment::Center);

        let query_tab = {
            let editor = text_editor(&self.query_input)
                .on_action(Message::QueryChanged)
                .highlight("graphql", self.highlighter_theme);
            let context_menu = ContextMenu::new(scrollable(editor), || {
                column![
                    button(row![lucide::copy().size(12), text(" Copy Query")].spacing(4))
                        .on_press(Message::CopyBody),
                    button(row![lucide::check().size(12), text(" Validate")].spacing(4))
                        .on_press(Message::ValidateQuery),
                ]
                .into()
            });

            let editor_with_autocomplete: Element<'_, Message, Theme, Renderer> =
                if self.autocomplete_suggestions.is_empty() {
                    container(context_menu)
                        .padding(5)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                } else {
                    let mut suggestions_list = column![].spacing(2);
                    for suggestion in &self.autocomplete_suggestions {
                        let s = suggestion.clone();
                        suggestions_list = suggestions_list.push(
                            button(text(suggestion.clone()).size(12))
                                .on_press(Message::AutocompleteSelected(s))
                                .width(Length::Fill),
                        );
                    }
                    let suggestions_popup = container(
                        scrollable(suggestions_list).height(Length::Fixed(150.0)),
                    )
                    .padding(4)
                    .style(move |_: &Theme| iced::widget::container::Style {
                        background: Some(iced::Color::from_rgb(0.15, 0.15, 0.2).into()),
                        border: iced::Border::default()
                            .rounded(4)
                            .color(iced::Color::from_rgb(0.3, 0.3, 0.4)),
                        ..iced::widget::container::Style::default()
                    });

                    column![context_menu, suggestions_popup]
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                };

            container(editor_with_autocomplete)
                .padding(5)
                .width(Length::Fill)
                .height(Length::Fill)
        };

        let variables_tab = {
            let editor = text_editor(&self.variables_input)
                .on_action(Message::VariablesChanged)
                .highlight("json", self.highlighter_theme);
            container(scrollable(editor))
                .padding(5)
                .width(Length::Fill)
                .height(Length::Fill)
        };

        let headers_tab = container(self.headers_editor.view().map(Message::HeadersEditor))
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill);

        let auth_tab = container(self.create_auth_tab_content())
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill);

        let tabs = Tabs::new(Message::TabSelected)
            .push(TabId::Query, TabLabel::Text("Query".to_string()), query_tab)
            .push(
                TabId::Variables,
                TabLabel::Text("Variables".to_string()),
                variables_tab,
            )
            .push(
                TabId::Headers,
                TabLabel::Text("Headers".to_string()),
                headers_tab,
            )
            .push(
                TabId::Authorization,
                TabLabel::Text("Authorization".to_string()),
                auth_tab,
            )
            .push(
                TabId::Schema,
                TabLabel::Text("Schema".to_string()),
                self.create_schema_tab(),
            )
            .set_active_tab(&self.active_tab)
            .width(Length::Fill)
            .height(Length::Fixed(250.0));

        let response_area: Element<Message> = match &self.request_status {
            RequestStatus::Idle => {
                let placeholder = if let Some(Err(e)) = &self.query_validation {
                    container(
                        column![
                            text("Enter query and send request.").size(14),
                            text(e.to_string())
                                .size(12)
                                .color(Color::from_rgb(0.8, 0.2, 0.2)),
                        ]
                        .spacing(5),
                    )
                } else {
                    container(text("Enter query and send request.").size(14))
                };
                placeholder
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .into()
            }
            RequestStatus::Loading => container(
                column![
                    iced_aw::Spinner::new().width(32).height(32),
                    text("Sending request...").size(14),
                ]
                .spacing(8)
                .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into(),
            RequestStatus::Success => {
                let response_tabs = Tabs::new(Message::ResponseTabSelected)
                    .push(ResponseTab::Body, TabLabel::Text("Body".to_string()), {
                        if self.word_wrap {
                            let body_text = self.response_body_editor.text();
                            let wrapped = text(body_text).size(13).font(iced::Font::MONOSPACE);
                            let context_menu = ContextMenu::new(scrollable(wrapped), || {
                                column![button(
                                    row![lucide::copy().size(12), text(" Copy Body")].spacing(4)
                                )
                                .on_press(Message::CopyBody)]
                                .into()
                            });
                            container(context_menu)
                        } else {
                            let editor = text_editor(&self.response_body_editor)
                                .on_action(Message::ResponseContentChanged)
                                .highlight("json", self.highlighter_theme);
                            let context_menu = ContextMenu::new(scrollable(editor), || {
                                column![
                                    button(
                                        row![lucide::copy().size(12), text(" Copy Selection")]
                                            .spacing(4)
                                    )
                                    .on_press(Message::CopySelection),
                                    button(
                                        row![lucide::copy().size(12), text(" Copy Body")]
                                            .spacing(4)
                                    )
                                    .on_press(Message::CopyBody),
                                ]
                                .into()
                            });
                            container(context_menu)
                        }
                    })
                    .push(
                        ResponseTab::Headers,
                        TabLabel::Text("Data".to_string()),
                        self.create_response_data_view(),
                    )
                    .set_active_tab(&self.active_response_tab)
                    .width(Length::Fill)
                    .height(Length::Fill);

                container(response_tabs)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
            RequestStatus::Error(error_message) => {
                container(text(format!("Error: {}", error_message)))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .into()
            }
        };

        let status_text = if let Some(status) = self.status_code {
            let color = status_color(status);
            text(format!("  {}  ", status)).size(14).color(color)
        } else {
            text(String::new()).size(14)
        };

        let save_status: Element<'_, Message, Theme, Renderer> =
            if let Some(msg) = &self.last_save_status {
                text(msg.clone())
                    .size(11)
                    .color(Color::from_rgb(0.2, 0.7, 0.3))
                    .into()
            } else {
                column![].into()
            };

        let duration_text = text(format!(
            "{}ms",
            self.response_duration
                .map(|d| d.as_millis().to_string())
                .unwrap_or_else(|| "N/A".to_string())
        ))
        .size(14);

        let size_text = text(
            self.response_size
                .map(|s| {
                    if s > 1024 {
                        format!("{:.1} KB", s as f64 / 1024.0)
                    } else {
                        format!("{} B", s)
                    }
                })
                .unwrap_or_else(|| "N/A".to_string()),
        )
        .size(14);

        let method_label = text("GraphQL")
            .size(14)
            .color(Color::from_rgb(0.8, 0.3, 0.6));

        let copy_button = if matches!(
            self.request_status,
            RequestStatus::Success | RequestStatus::Error(_)
        ) {
            Element::from(
                button(row![lucide::copy().size(14), text(" Copy")].spacing(4))
                    .on_press(Message::CopyResponse),
            )
        } else {
            Element::from(column![])
        };

        let wrap_toggle: Element<'_, Message, Theme, Renderer> =
            if matches!(self.request_status, RequestStatus::Success) {
                Element::from(
                    button(
                        row![
                            lucide::wrap_text().size(14),
                            text(if self.word_wrap {
                                "Wrap ON"
                            } else {
                                "Wrap OFF"
                            })
                            .size(11),
                        ]
                        .spacing(4),
                    )
                    .on_press(Message::ToggleWordWrap),
                )
            } else {
                Element::from(column![])
            };

        let validation_indicator: Element<'_, Message, Theme, Renderer> =
            match &self.query_validation {
                Some(Ok(())) => text("Valid")
                    .size(12)
                    .color(Color::from_rgb(0.2, 0.7, 0.3))
                    .into(),
                Some(Err(e)) => text(e.to_string())
                    .size(12)
                    .color(Color::from_rgb(0.8, 0.2, 0.2))
                    .into(),
                None => column![].into(),
            };

        let operation_name_display: Element<'_, Message, Theme, Renderer> =
            if self.operation_name.is_empty() {
                column![].into()
            } else {
                text(format!("Op: {}", self.operation_name))
                    .size(12)
                    .color(Color::from_rgb(0.5, 0.5, 0.5))
                    .into()
            };

        let main_column = column![
            url_bar,
            row![
                text_input("Operation name (optional)", &self.operation_name)
                    .on_input(Message::OperationNameChanged)
                    .padding(8)
                    .width(Length::Fixed(200.0)),
                validation_indicator,
                operation_name_display,
            ]
            .spacing(10)
            .padding(10)
            .align_y(Alignment::Center),
            tabs,
            rule::horizontal(10),
            column![
                    row![
                        method_label,
                        status_text,
                        duration_text,
                        text(" | ").size(14),
                        size_text,
                        save_status,
                        row![copy_button, wrap_toggle].align_y(Alignment::Center),
                    ]
                .spacing(10)
                .padding(10)
                .align_y(Alignment::Center),
                response_area,
            ]
            .height(Length::Fill),
        ]
        .align_x(Alignment::Center);

        scrollable(main_column).into()
    }

    fn create_schema_tab(&self) -> Element<'_, Message, Theme, Renderer> {
        let search_bar = text_input("Search types...", &self.schema_search)
            .on_input(Message::SchemaSearchChanged)
            .padding(8);

        let introspect_button = if self.schema_loading {
            button(text("Loading schema...").size(12))
        } else {
            button(row![lucide::refresh_cw().size(12), text(" Fetch Schema")].spacing(4))
                .on_press(Message::IntrospectSchema)
        };

        let header = row![search_bar, introspect_button]
            .spacing(10)
            .align_y(Alignment::Center);

        let content: Element<Message> = if self.schema_loading {
            container(text("Loading schema...").size(14))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into()
        } else if let Some(schema) = &self.schema {
            let mut types_list = column![].spacing(2);

            let filtered_types: Vec<&SchemaType> = if self.schema_search.is_empty() {
                schema.types.iter().collect()
            } else {
                let search_lower = self.schema_search.to_lowercase();
                schema
                    .types
                    .iter()
                    .filter(|t| t.name.to_lowercase().contains(&search_lower))
                    .collect()
            };

            for schema_type in &filtered_types {
                let kind_color = match schema_type.kind {
                    TypeKind::Object => Color::from_rgb(0.2, 0.6, 0.9),
                    TypeKind::Interface => Color::from_rgb(0.5, 0.3, 0.8),
                    TypeKind::Enum => Color::from_rgb(0.9, 0.6, 0.1),
                    TypeKind::InputObject => Color::from_rgb(0.3, 0.7, 0.4),
                    TypeKind::Scalar => Color::from_rgb(0.6, 0.6, 0.6),
                    TypeKind::Union => Color::from_rgb(0.8, 0.4, 0.6),
                    _ => Color::from_rgb(0.5, 0.5, 0.5),
                };

                let type_label = row![
                    text(format!("[{}]", schema_type.kind)).size(11).color(kind_color),
                    text(&schema_type.name).size(13),
                ]
                .spacing(5);

                let is_selected = self
                    .schema_selected_type
                    .as_ref()
                    .map(|s| s == &schema_type.name)
                    .unwrap_or(false);

                let item = if is_selected {
                    button(type_label).on_press(Message::SchemaTypeSelected(String::new()))
                } else {
                    button(type_label)
                        .on_press(Message::SchemaTypeSelected(schema_type.name.clone()))
                };

                types_list = types_list.push(item.padding(4));
            }

            let type_list_scroll = scrollable(types_list).height(Length::Fill);

            let detail_panel: Element<Message> = if let Some(selected_name) =
                &self.schema_selected_type
            {
                if let Some(selected_type) = schema.types.iter().find(|t| &t.name == selected_name)
                {
                    let mut detail = column![].spacing(5);

                    detail = detail.push(
                        row![
                            text(format!("[{}]", selected_type.kind))
                                .size(14)
                                .color(Color::from_rgb(0.2, 0.6, 0.9)),
                            text(&selected_type.name).size(16),
                        ]
                        .spacing(8),
                    );

                    if let Some(desc) = &selected_type.description {
                        if !desc.is_empty() {
                            detail = detail.push(text(desc.clone()).size(12).color(
                                Color::from_rgb(0.6, 0.6, 0.6),
                            ));
                        }
                    }

                    if !selected_type.fields.is_empty() {
                        detail = detail.push(rule::horizontal(5));
                        detail = detail.push(text("Fields:").size(13).color(
                            Color::from_rgb(0.5, 0.5, 0.5),
                        ));
                        for field in &selected_type.fields {
                            let mut field_text = row![
                                text(&field.name).size(12),
                                text(": ").size(12).color(Color::from_rgb(0.5, 0.5, 0.5)),
                                text(&field.return_type).size(12).color(Color::from_rgb(
                                    0.2, 0.6, 0.9,
                                )),
                            ]
                            .spacing(4);

                            if field.is_deprecated {
                                field_text = field_text.push(
                                    text(" [deprecated]")
                                        .size(10)
                                        .color(Color::from_rgb(0.8, 0.4, 0.1)),
                                );
                            }

                            if !field.args.is_empty() {
                                let args_str: Vec<String> = field
                                    .args
                                    .iter()
                                    .map(|a| format!("{}: {}", a.name, a.arg_type))
                                    .collect();
                                field_text = field_text.push(
                                    text(format!("({})", args_str.join(", ")))
                                        .size(10)
                                        .color(Color::from_rgb(0.5, 0.5, 0.5)),
                                );
                            }

                            detail = detail.push(field_text);
                        }
                    }

                    if !selected_type.input_fields.is_empty() {
                        detail = detail.push(rule::horizontal(5));
                        detail = detail.push(text("Input Fields:").size(13).color(
                            Color::from_rgb(0.5, 0.5, 0.5),
                        ));
                        for field in &selected_type.input_fields {
                            detail = detail.push(
                                row![
                                    text(&field.name).size(12),
                                    text(": ").size(12).color(Color::from_rgb(0.5, 0.5, 0.5)),
                                    text(&field.field_type).size(12).color(Color::from_rgb(
                                        0.2, 0.6, 0.9,
                                    )),
                                ]
                                .spacing(4),
                            );
                        }
                    }

                    if !selected_type.enum_values.is_empty() {
                        detail = detail.push(rule::horizontal(5));
                        detail = detail.push(text("Enum Values:").size(13).color(
                            Color::from_rgb(0.5, 0.5, 0.5),
                        ));
                        for val in &selected_type.enum_values {
                            let mut val_text = row![text(&val.name).size(12)].spacing(4);
                            if val.is_deprecated {
                                val_text = val_text.push(
                                    text(" [deprecated]")
                                        .size(10)
                                        .color(Color::from_rgb(0.8, 0.4, 0.1)),
                                );
                            }
                            if let Some(desc) = &val.description {
                                if !desc.is_empty() {
                                    val_text = val_text.push(
                                        text(format!("- {}", desc))
                                            .size(10)
                                            .color(Color::from_rgb(0.5, 0.5, 0.5)),
                                    );
                                }
                            }
                            detail = detail.push(val_text);
                        }
                    }

                    if !selected_type.interfaces.is_empty() {
                        detail = detail.push(rule::horizontal(5));
                        detail = detail.push(
                            text(format!("Implements: {}", selected_type.interfaces.join(", ")))
                                .size(12),
                        );
                    }

                    container(scrollable(detail))
                        .padding(10)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                } else {
                    container(text("Type not found"))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                        .into()
                }
            } else if let Some(qt) = &schema.query_type {
                container(
                    column![
                        text("Select a type from the list to view its details.").size(12),
                        text(format!("Query root: {}", qt)).size(12).color(
                            Color::from_rgb(0.2, 0.6, 0.9),
                        ),
                        {
                            let mut info = column![].spacing(2);
                            if let Some(mt) = &schema.mutation_type {
                                info = info.push(
                                    text(format!("Mutation root: {}", mt)).size(12).color(
                                        Color::from_rgb(0.2, 0.6, 0.9),
                                    ),
                                );
                            }
                            if let Some(st) = &schema.subscription_type {
                                info = info.push(
                                    text(format!("Subscription root: {}", st)).size(12).color(
                                        Color::from_rgb(0.2, 0.6, 0.9),
                                    ),
                                );
                            }
                            info
                        },
                    ]
                    .spacing(5),
                )
                .padding(10)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            } else {
                container(text("Schema loaded but no query type found."))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .into()
            };

            row![
                container(type_list_scroll).width(Length::FillPortion(1)).height(Length::Fill),
                detail_panel
            ]
                .spacing(10)
                .into()
        } else {
            container(
                column![
                    text("No schema loaded.").size(14),
                    text("Click 'Fetch Schema' to introspect the GraphQL endpoint.")
                        .size(12)
                        .color(Color::from_rgb(0.5, 0.5, 0.5)),
                ]
                .spacing(5)
                .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
        };

        container(column![header, rule::horizontal(5), content].spacing(5))
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn create_auth_tab_content(&self) -> Element<'_, Message, Theme, Renderer> {
        let oauth2_content = column![
            text("OAuth2 not fully supported for GraphQL yet").size(12),
            text("Use Bearer token or manually configure").size(12),
        ]
        .spacing(10)
        .into();

        crate::ui::components::auth_panel::auth_panel(
            &self.auth,
            Message::AuthTypeSelected,
            |t| Message::AuthInputChanged(AuthInput::BearerToken(t)),
            |u| Message::AuthInputChanged(AuthInput::BasicUser(u)),
            |p| Message::AuthInputChanged(AuthInput::BasicPass(p)),
            |k| Message::AuthInputChanged(AuthInput::ApiKeyKey(k)),
            |v| Message::AuthInputChanged(AuthInput::ApiKeyValue(v)),
            |loc| Message::AuthInputChanged(AuthInput::ApiKeyLocation(loc)),
            |u| Message::AuthInputChanged(AuthInput::DigestUser(u)),
            |p| Message::AuthInputChanged(AuthInput::DigestPass(p)),
            oauth2_content,
        )
    }

    fn create_response_data_view(&self) -> Element<'_, Message, Theme, Renderer> {
        if let Some(response) = &self.last_response {
            let mut items = column![].spacing(8);

            if let Some(data) = &response.data {
                let data_str = serde_json::to_string_pretty(data).unwrap_or_default();
                items = items.push(
                    row![text("Data:").size(14).color(Color::from_rgb(0.5, 0.5, 0.5)),].spacing(8),
                );
                items = items.push(text(data_str).size(13).font(iced::Font::MONOSPACE));
            }

            if !response.errors.is_empty() {
                items = items.push(rule::horizontal(5));
                items = items.push(
                    text(format!("Errors ({}):", response.errors.len()))
                        .size(14)
                        .color(Color::from_rgb(0.8, 0.3, 0.3)),
                );
                for (i, err) in response.errors.iter().enumerate() {
                    items = items.push(
                        text(format!("{}. {}", i + 1, err.message))
                            .size(13)
                            .color(Color::from_rgb(0.8, 0.3, 0.3)),
                    );
                }
            }

            container(scrollable(items))
                .padding(10)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            container(text("No response data available."))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into()
        }
    }
}
