use crate::persistence::database::{self, Environment};
use crate::protocols::websocket::{WsEvent, WsSender, WsStatus};
use crate::ui::toast::ToastManager;
use crate::ui::views::collection_view::{self, CollectionView};
use crate::ui::views::environment_manager::{self, EnvironmentManagerView};
use crate::ui::views::history_view::{self, HistoryView};
use crate::ui::views::websocket_view::{self, WebSocketView};
use iced::{
    widget::{button, column, container, pick_list, row, rule, stack, text},
    Alignment, Element, Length, Subscription, Task,
};
use iced_aw::{TabLabel, Tabs};
use iced_fonts::lucide;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use super::views::graphql_view::{self, GraphQLView};
use super::views::http_request_view::{self, HttpRequestView};

use iced::futures::stream::BoxStream;
use iced::futures::{self, StreamExt as _};
use iced_futures::subscription::{from_recipe, EventStream, Recipe};

struct WsRecipe {
    receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<WsEvent>>>>,
    connection_id: u64,
}

impl Recipe for WsRecipe {
    type Output = Message;

    fn hash(&self, state: &mut iced_futures::subscription::Hasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<WsRecipe>().hash(state);
        self.connection_id.hash(state);
    }

    fn stream(self: Box<Self>, _input: EventStream) -> BoxStream<'static, Message> {
        let receiver_arc = self.receiver;
        futures::stream::unfold(receiver_arc, |arc| async move {
            // Take the receiver out of the Option temporarily
            let mut receiver = {
                let mut guard = arc.lock().ok()?;
                guard.take()?
            };
            // Await outside the lock so we don't hold MutexGuard across await
            let event = receiver.recv().await?;
            // Put the receiver back
            if let Ok(mut guard) = arc.lock() {
                *guard = Some(receiver);
            }
            Some((Message::WsEvent(event), arc))
        })
        .boxed()
    }
}

struct DevicePollRecipe {
    tab_index: usize,
    device_code: String,
    client_id: String,
    client_secret: String,
    token_url: String,
    interval_secs: u64,
    http_client: Arc<reqwest::Client>,
}

impl Recipe for DevicePollRecipe {
    type Output = Message;

    fn hash(&self, state: &mut iced_futures::subscription::Hasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<DevicePollRecipe>().hash(state);
        self.tab_index.hash(state);
        self.device_code.hash(state);
    }

    fn stream(self: Box<Self>, _input: EventStream) -> BoxStream<'static, Message> {
        let tab_index = self.tab_index;
        let device_code = self.device_code;
        let client_id = self.client_id;
        let client_secret = self.client_secret;
        let token_url = self.token_url;
        let interval = std::time::Duration::from_secs(self.interval_secs.max(5));
        let http_client = self.http_client;

        futures::stream::unfold((), move |()| {
            let device_code = device_code.clone();
            let client_id = client_id.clone();
            let client_secret = client_secret.clone();
            let token_url = token_url.clone();
            let http_client = http_client.clone();
            async move {
                tokio::time::sleep(interval).await;
                let result = crate::data::oauth2::poll_device_token(
                    &http_client,
                    &token_url,
                    &device_code,
                    &client_id,
                    &client_secret,
                )
                .await;
                Some((Message::OAuth2DeviceTokenPoll(tab_index, result), ()))
            }
        })
        .boxed()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Http,
    WebSocket,
    GraphQL,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Http => write!(f, "HTTP"),
            Protocol::WebSocket => write!(f, "WebSocket"),
            Protocol::GraphQL => write!(f, "GraphQL"),
        }
    }
}

impl Protocol {
    pub const ALL: [Protocol; 3] = [Protocol::Http, Protocol::WebSocket, Protocol::GraphQL];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Main,
    EnvironmentManager,
}

pub fn main() -> iced::Result {
    iced::application(AstraNovaApp::new, AstraNovaApp::update, AstraNovaApp::view)
        .title("AstraNova Client")
        .subscription(AstraNovaApp::subscription)
        .theme(AstraNovaApp::theme)
        .font(iced_fonts::LUCIDE_FONT_BYTES)
        .run()
}

pub(crate) struct AstraNovaApp {
    pub(crate) request_tabs: Vec<HttpRequestView>,
    pub(crate) active_request_tab_index: usize,
    pub(crate) http_client: Arc<reqwest::Client>,
    pub(crate) custom_clients: HashMap<String, Arc<reqwest::Client>>,
    pub(crate) db_conn: rusqlite::Connection,
    pub(crate) environments: Vec<Environment>,
    pub(crate) active_environment: Option<Environment>,
    pub(crate) env_manager_view: EnvironmentManagerView,
    pub(crate) history_view: HistoryView,
    pub(crate) collection_view: CollectionView,
    pub(crate) websocket_view: WebSocketView,
    pub(crate) graphql_view: GraphQLView,
    pub(crate) active_protocol: Protocol,
    pub(crate) current_view: View,
    pub(crate) show_history: bool,
    pub(crate) show_collections: bool,
    pub(crate) show_env_info: bool,
    pub(crate) ws_sender: Option<WsSender>,
    pub(crate) ws_receiver: Option<Arc<Mutex<Option<mpsc::UnboundedReceiver<WsEvent>>>>>,
    pub(crate) ws_shutdown: Option<mpsc::UnboundedSender<()>>,
    pub(crate) ws_write_handle: Option<Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>>,
    pub(crate) ws_read_handle: Option<Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>>,
    pub(crate) ws_connection_id: u64,
    pub(crate) toast_manager: ToastManager,
    pub(crate) dark_mode: bool,
    pub(crate) secret_store: crate::services::secret_store::SecretStore,
}

#[derive(Debug, Clone)]
pub enum Message {
    HttpRequestViewMsg(usize, http_request_view::Message),
    AddRequestTab,
    CloseRequestTab(usize),
    CloseActiveRequestTab,
    NoOp,
    SelectRequestTab(usize),
    PrevRequestTab,
    NextRequestTab,
    EnvManagerMsg(environment_manager::Message),
    EnvFileLoaded(Option<Vec<(String, String)>>),
    EnvFileExported(Option<String>),
    SelectEnvironment(i32),
    SwitchView(View),
    ToggleEnvironmentManager,
    HistoryMsg(history_view::Message),
    HistoryExportComplete(Option<String>),
    ToggleHistory,
    CollectionMsg(collection_view::Message),
    ToggleCollections,
    ToggleEnvInfo,
    ToggleTheme,
    WebSocketMsg(websocket_view::Message),
    GraphQLMsg(graphql_view::Message),
    WsEvent(crate::protocols::websocket::WsEvent),
    WsConnected(
        WsSender,
        Arc<Mutex<Option<mpsc::UnboundedReceiver<WsEvent>>>>,
        Option<mpsc::UnboundedSender<()>>,
        Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
        Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    ),
    SelectProtocol(Protocol),
    OAuth2StartAuth(usize),
    OAuth2AuthComplete(
        usize,
        Result<String, crate::error::AppError>,
        Option<String>,
    ),
    OAuth2TokenReceived(
        usize,
        Result<crate::data::oauth2::OAuth2TokenResponse, crate::error::AppError>,
    ),
    OAuth2RefreshToken(usize),
    OAuth2StartDeviceAuth(usize),
    OAuth2DeviceAuthReceived(
        usize,
        Result<crate::data::oauth2::DeviceAuthorizationResponse, crate::error::AppError>,
    ),
    OAuth2DeviceTokenPoll(
        usize,
        Result<crate::data::oauth2::DeviceTokenResponse, crate::error::AppError>,
    ),
    OAuth2AutoPollToggle(usize, bool),
    ToggleResponseSearch,
    WsSendFromKeyboard,
    ClearKeychainSecrets,
    KeychainCleared(Result<u32, crate::error::AppError>),
}

impl AstraNovaApp {
    fn new() -> (Self, Task<Message>) {
        let (db_conn, environments) = match database::init() {
            Ok(conn) => {
                let envs = crate::services::environment_service::get_all(&conn)
                    .unwrap_or_else(|e| {
                        log::error!("Failed to load environments: {}", e);
                        Vec::new()
                    });
                (conn, envs)
            }
            Err(e) => {
                log::error!("Failed to initialize database: {}", e);
                let conn = rusqlite::Connection::open_in_memory()
                    .expect("In-memory DB should always work");
                if let Err(schema_err) = database::init_schema(&conn) {
                    log::error!("Failed to init in-memory schema: {}", schema_err);
                }
                (conn, Vec::new())
            }
        };

        let history = crate::services::history_service::get_all(&db_conn, 200)
            .unwrap_or_else(|e| {
                log::error!("Failed to load history: {}", e);
                Vec::new()
            });
        let collections = crate::services::collection_service::get_all(&db_conn)
            .unwrap_or_else(|e| {
                log::error!("Failed to load collections: {}", e);
                Vec::new()
            });

        let mut cv = CollectionView::new();
        cv.sync_collections(&collections);

        let secret_store = crate::services::secret_store::SecretStore::new();
        match crate::services::secret_store::migrate_plaintext_tokens_to_keyring(&secret_store, &db_conn) {
            Ok(0) => {}
            Ok(n) => log::info!("Migrated {} plaintext tokens to OS keyring", n),
            Err(e) => log::warn!("Keyring migration skipped: {}", e),
        }

        let app = Self {
            request_tabs: vec![HttpRequestView::default()],
            active_request_tab_index: 0,
            http_client: Arc::new(reqwest::Client::new()),
            custom_clients: HashMap::new(),
            db_conn,
            environments: environments.clone(),
            active_environment: None,
            env_manager_view: EnvironmentManagerView::new(environments),
            history_view: {
                let mut hv = HistoryView::new();
                hv.entries = history;
                hv
            },
            collection_view: cv,
            websocket_view: WebSocketView::new(),
            graphql_view: GraphQLView::default(),
            active_protocol: Protocol::Http,
            current_view: View::Main,
            show_history: false,
            show_collections: false,
            show_env_info: false,
            ws_sender: None,
            ws_receiver: None,
            ws_shutdown: None,
            ws_write_handle: None,
            ws_read_handle: None,
            ws_connection_id: 0,
            toast_manager: ToastManager::new(),
            dark_mode: true,
            secret_store,
        };
        (app, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        self.toast_manager.clean_expired();
        match message {
            Message::HttpRequestViewMsg(index, msg) => {
                super::handlers::http_request::handle_http_request_msg(self, index, msg)
            }
            Message::AddRequestTab => {
                let mut new_view = HttpRequestView::default();
                if let Some(env) = &self.active_environment {
                    if let Some(url) = &env.default_endpoint {
                        if !url.is_empty() {
                            new_view.url_input = url.clone();
                        }
                    }
                }
                self.request_tabs.push(new_view);
                self.active_request_tab_index = self.request_tabs.len() - 1;
                Task::none()
            }
            Message::CloseRequestTab(index) => {
                if self.request_tabs.len() > 1 {
                    self.request_tabs.remove(index);
                    if self.active_request_tab_index >= self.request_tabs.len() {
                        self.active_request_tab_index = self.request_tabs.len() - 1;
                    }
                }
                Task::none()
            }
            Message::CloseActiveRequestTab => {
                if self.request_tabs.len() > 1 {
                    let index = self.active_request_tab_index;
                    self.request_tabs.remove(index);
                    if self.active_request_tab_index >= self.request_tabs.len() {
                        self.active_request_tab_index = self.request_tabs.len() - 1;
                    }
                }
                Task::none()
            }
            Message::NoOp => Task::none(),
            Message::SelectRequestTab(index) => {
                self.active_request_tab_index = index;
                Task::none()
            }
            Message::PrevRequestTab => {
                if !self.request_tabs.is_empty() {
                    self.active_request_tab_index =
                        (self.active_request_tab_index + self.request_tabs.len() - 1)
                            % self.request_tabs.len();
                }
                Task::none()
            }
            Message::NextRequestTab => {
                if !self.request_tabs.is_empty() {
                    self.active_request_tab_index =
                        (self.active_request_tab_index + 1) % self.request_tabs.len();
                }
                Task::none()
            }
            Message::EnvManagerMsg(msg) => super::handlers::environment::handle_message(self, msg),
            Message::EnvFileLoaded(vars) => {
                super::handlers::environment::handle_file_loaded(self, vars)
            }
            Message::EnvFileExported(content) => {
                if let Some(content) = content {
                    self.toast_manager
                        .success(format!("Exported .env file ({} bytes)", content.len()));
                }
                Task::none()
            }
            Message::SelectEnvironment(id) => {
                self.active_environment = self.environments.iter().find(|e| e.id == id).cloned();
                Task::none()
            }
            Message::SwitchView(view) => {
                self.current_view = view;
                Task::none()
            }
            Message::ToggleEnvironmentManager => {
                self.current_view = match self.current_view {
                    View::EnvironmentManager => View::Main,
                    View::Main => View::EnvironmentManager,
                };
                Task::none()
            }
            Message::ToggleHistory => {
                self.show_history = !self.show_history;
                Task::none()
            }
            Message::ToggleCollections => {
                self.show_collections = !self.show_collections;
                if self.show_collections {
                    let cols = crate::services::collection_service::get_all(&self.db_conn)
                        .unwrap_or_else(|e| {
                            log::error!("Failed to refresh collections: {}", e);
                            Vec::new()
                        });
                    self.collection_view.sync_collections(&cols);
                }
                Task::none()
            }
            Message::ToggleEnvInfo => {
                self.show_env_info = !self.show_env_info;
                Task::none()
            }
            Message::ToggleTheme => {
                self.dark_mode = !self.dark_mode;
                Task::none()
            }
            Message::CollectionMsg(msg) => super::handlers::collection::handle_message(self, msg),
            Message::HistoryMsg(msg) => super::handlers::history::handle_message(self, msg),
            Message::HistoryExportComplete(result) => {
                if let Some(msg) = result {
                    if msg.contains("failed") || msg.contains("cancelled") {
                        self.toast_manager.warning(msg);
                    } else {
                        self.toast_manager.success(msg);
                    }
                }
                Task::none()
            }
            Message::SelectProtocol(protocol) => {
                self.active_protocol = protocol;
                Task::none()
            }
            Message::WsEvent(event) => super::handlers::websocket::handle_ws_event(self, event),
            Message::WebSocketMsg(msg) => super::handlers::websocket::handle_message(self, msg),
            Message::GraphQLMsg(msg) => super::handlers::graphql::handle_message(self, msg),
            Message::WsConnected(sender, receiver_arc, shutdown_tx, write_handle, read_handle) => {
                super::handlers::websocket::handle_ws_connected(
                    self,
                    sender,
                    receiver_arc,
                    shutdown_tx,
                    write_handle,
                    read_handle,
                );
                Task::none()
            }
            Message::OAuth2StartAuth(index) => {
                super::handlers::oauth2::handle_start_auth(self, index)
            }
            Message::OAuth2AuthComplete(index, result, pkce_verifier) => {
                super::handlers::oauth2::handle_auth_complete(self, index, result, pkce_verifier)
            }
            Message::OAuth2TokenReceived(index, result) => {
                super::handlers::oauth2::handle_token_received(self, index, result)
            }
            Message::OAuth2RefreshToken(index) => {
                super::handlers::oauth2::handle_refresh_token(self, index)
            }
            Message::OAuth2StartDeviceAuth(index) => {
                super::handlers::oauth2::handle_start_device_auth(self, index)
            }
            Message::OAuth2DeviceAuthReceived(index, result) => {
                super::handlers::oauth2::handle_device_auth_received(self, index, result)
            }
            Message::OAuth2DeviceTokenPoll(index, result) => {
                super::handlers::oauth2::handle_device_token_poll(self, index, result)
            }
            Message::OAuth2AutoPollToggle(index, enabled) => {
                super::handlers::oauth2::handle_auto_poll_toggle(self, index, enabled)
            }
            Message::ToggleResponseSearch => {
                if let Some(view) = self.request_tabs.get_mut(self.active_request_tab_index) {
                    view.update(http_request_view::Message::ToggleResponseSearch);
                }
                Task::none()
            }
            Message::WsSendFromKeyboard => {
                if self.active_protocol == Protocol::WebSocket {
                    let input = self.websocket_view.input.clone();
                    if !input.is_empty()
                        && matches!(self.websocket_view.status, WsStatus::Connected)
                    {
                        if let Some(sender) = &self.websocket_view.ws_sender {
                            let _ = sender.send(&input);
                            self.websocket_view
                                .messages
                                .push(crate::protocols::websocket::WsMessage::outgoing(input));
                            self.websocket_view.input.clear();
                        }
                    }
                }
                Task::none()
            }
            Message::ClearKeychainSecrets => {
                let store = self.secret_store.clone();
                let conn = &self.db_conn;
                let mut identifiers = Vec::new();

                if let Ok(mut stmt) = conn.prepare(
                    "SELECT id, collection_id FROM collection_requests WHERE auth_type = 'oauth2'"
                ) {
                    if let Ok(rows) = stmt.query_map([], |row| Ok((row.get::<_, i32>(0)?, row.get::<_, i32>(1)?))) {
                        for row in rows.flatten() {
                            identifiers.push(format!("col_{}_{}", row.1, row.0));
                        }
                    }
                }

                if let Ok(mut stmt) = conn.prepare(
                    "SELECT id FROM request_history WHERE request_data LIKE '%OAuth2%'"
                ) {
                    if let Ok(rows) = stmt.query_map([], |row| row.get::<_, i32>(0)) {
                        for row in rows.flatten() {
                            identifiers.push(format!("hist_{}", row));
                        }
                    }
                }

                Task::perform(
                    async move {
                        let mut total = 0u32;
                        for identifier in &identifiers {
                            let _ = store.delete_oauth2_tokens(identifier);
                            total += 1;
                        }
                        total
                    },
                    |count| Message::KeychainCleared(Ok(count)),
                )
            }
            Message::KeychainCleared(result) => {
                match result {
                    Ok(count) => {
                        self.toast_manager.success(format!(
                            "Cleared {} keychain entries",
                            count
                        ));
                    }
                    Err(e) => {
                        self.toast_manager
                            .error(format!("Failed to clear keychain: {}", e));
                    }
                }
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let ws_subscription = if let Some(receiver_arc) = &self.ws_receiver {
            from_recipe(WsRecipe {
                receiver: receiver_arc.clone(),
                connection_id: self.ws_connection_id,
            })
        } else {
            Subscription::none()
        };

        let keyboard_subscription = iced::event::listen_with(|event, _status, _window| {
            if let iced::event::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) = event {
                if modifiers.control() || modifiers.command() {
                    match key {
                        iced::keyboard::Key::Character(ref c) if c.as_ref() == "n" => {
                            Some(Message::AddRequestTab)
                        }
                        iced::keyboard::Key::Character(ref c) if c.as_ref() == "w" => {
                            Some(Message::CloseActiveRequestTab)
                        }
                        iced::keyboard::Key::Character(ref c) if c.as_ref() == "t" => {
                            Some(Message::AddRequestTab)
                        }
                        iced::keyboard::Key::Character(ref c) if c.as_ref() == "d" => {
                            Some(Message::ToggleTheme)
                        }
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowLeft) => {
                            Some(Message::PrevRequestTab)
                        }
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowRight) => {
                            Some(Message::NextRequestTab)
                        }
                        iced::keyboard::Key::Character(ref c) if c.as_ref() == "1" => {
                            Some(Message::SelectRequestTab(0))
                        }
                        iced::keyboard::Key::Character(ref c) if c.as_ref() == "2" => {
                            Some(Message::SelectRequestTab(1))
                        }
                        iced::keyboard::Key::Character(ref c) if c.as_ref() == "3" => {
                            Some(Message::SelectRequestTab(2))
                        }
                        iced::keyboard::Key::Character(ref c) if c.as_ref() == "4" => {
                            Some(Message::SelectRequestTab(3))
                        }
                        iced::keyboard::Key::Character(ref c) if c.as_ref() == "5" => {
                            Some(Message::SelectRequestTab(4))
                        }
                        iced::keyboard::Key::Character(ref c) if c.as_ref() == "f" => {
                            Some(Message::ToggleResponseSearch)
                        }
                        iced::keyboard::Key::Character(ref c) if c.as_ref() == "e" => {
                            Some(Message::ToggleEnvironmentManager)
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        });

        let device_poll_subscription = self.device_poll_subscription();

        Subscription::batch(vec![
            ws_subscription,
            keyboard_subscription,
            device_poll_subscription,
        ])
    }

    fn device_poll_subscription(&self) -> Subscription<Message> {
        for (index, tab) in self.request_tabs.iter().enumerate() {
            if let crate::data::auth::Auth::OAuth2(config) = &tab.auth {
                if config.auto_polling
                    && !config.device_code.is_empty()
                    && !config.token_url.is_empty()
                {
                    let interval = config.device_code_interval.unwrap_or(5);
                    return from_recipe(DevicePollRecipe {
                        tab_index: index,
                        device_code: config.device_code.clone(),
                        client_id: config.client_id.clone(),
                        client_secret: config.client_secret.clone(),
                        token_url: config.token_url.clone(),
                        interval_secs: interval,
                        http_client: self.http_client.clone(),
                    });
                }
            }
        }
        Subscription::none()
    }

    fn theme(&self) -> iced::Theme {
        if self.dark_mode {
            iced::Theme::Dark
        } else {
            iced::Theme::Light
        }
    }

    fn create_toolbar(&self) -> (Element<'_, Message>, Element<'_, Message>) {
        let add_tab_button = button(lucide::plus().size(16)).on_press(Message::AddRequestTab);
        let close_tab_button = if self.request_tabs.len() > 1 {
            button(lucide::x().size(16))
                .on_press(Message::CloseRequestTab(self.active_request_tab_index))
        } else {
            button(lucide::x().size(16))
        };

        let history_button = button(row![lucide::history().size(14), text(" History")].spacing(4))
            .on_press(Message::ToggleHistory);

        let collections_button =
            button(row![lucide::folder().size(14), text(" Collections")].spacing(4))
                .on_press(Message::ToggleCollections);

        let theme_button = if self.dark_mode {
            button(row![lucide::sun().size(14), text(" Light")].spacing(4))
                .on_press(Message::ToggleTheme)
        } else {
            button(row![lucide::moon().size(14), text(" Dark")].spacing(4))
                .on_press(Message::ToggleTheme)
        };

        let protocol_selector = pick_list(
            &Protocol::ALL[..],
            Some(self.active_protocol),
            Message::SelectProtocol,
        );

        let env_selector = pick_list(
            &self.environments[..],
            self.active_environment.clone(),
            |env| Message::SelectEnvironment(env.id),
        )
        .placeholder("No Environment");

        let mut env_controls = row![
            history_button,
            collections_button,
            theme_button,
            protocol_selector,
            env_selector,
            button(row![lucide::settings().size(14), text(" Manage Environments")].spacing(4))
                .on_press(Message::SwitchView(View::EnvironmentManager))
        ]
        .spacing(10);

        if self.active_environment.is_some() {
            let chevron = if self.show_env_info {
                lucide::chevron_down().size(12)
            } else {
                lucide::chevron_right().size(12)
            };
            env_controls = env_controls.push(
                button(row![chevron, text(" Help").size(12)].spacing(4))
                    .on_press(Message::ToggleEnvInfo),
            );
        }

        let toolbar = row![add_tab_button, close_tab_button, env_controls]
            .spacing(10)
            .padding(10)
            .align_y(Alignment::Center);

        let env_help_section: Element<Message> = if let Some(active_env) = &self.active_environment
        {
            if self.show_env_info {
                let variables_text = if active_env.variables.is_empty() {
                    "This environment has no variables.".to_string()
                } else {
                    let keys: Vec<_> = active_env
                        .variables
                        .iter()
                        .map(|(k, _)| k.as_str())
                        .collect();
                    format!("Available: {}", keys.join(", "))
                };
                column![
                    text("Use {{variable}} in URL, Headers, or Body.").size(12),
                    text(variables_text).size(12)
                ]
                .spacing(5)
                .into()
            } else {
                column![].into()
            }
        } else {
            column![].into()
        };

        (toolbar.into(), env_help_section)
    }

    fn view(&self) -> Element<'_, Message> {
        match self.current_view {
            View::Main => {
                let mut tabs = Tabs::new(Message::SelectRequestTab);

                for (index, request_tab) in self.request_tabs.iter().enumerate() {
                    let tab_label = if request_tab.url_input.is_empty() {
                        TabLabel::Text(format!("New Request {}", index + 1))
                    } else {
                        let url = request_tab.url_input.chars().take(25).collect::<String>();
                        let truncated_url = if request_tab.url_input.len() > 25 {
                            format!("{}...", url)
                        } else {
                            url
                        };
                        TabLabel::Text(format!("{} {}", request_tab.method, truncated_url))
                    };

                    tabs = tabs.push(
                        index,
                        tab_label,
                        request_tab
                            .view()
                            .map(move |msg| Message::HttpRequestViewMsg(index, msg)),
                    );
                }

                let tabs_widget = tabs
                    .set_active_tab(&self.active_request_tab_index)
                    .width(Length::Fill)
                    .height(Length::Fill);

                let (toolbar, env_help_section) = self.create_toolbar();

                let main_content = match self.active_protocol {
                    Protocol::Http => {
                        column![toolbar, env_help_section, tabs_widget,]
                    }
                    Protocol::WebSocket => {
                        column![
                            toolbar,
                            env_help_section,
                            self.websocket_view.view().map(Message::WebSocketMsg),
                        ]
                    }
                    Protocol::GraphQL => {
                        column![
                            toolbar,
                            env_help_section,
                            self.graphql_view.view().map(Message::GraphQLMsg),
                        ]
                    }
                };

                let toast_overlay = self.toast_manager.view().map(|_| Message::NoOp);

                let content: Element<'_, Message> = if self.show_history {
                    let history_panel =
                        container(self.history_view.view().map(Message::HistoryMsg))
                            .width(Length::FillPortion(1))
                            .height(Length::Fill);

                    if self.show_collections {
                        let collections_panel =
                            container(self.collection_view.view().map(Message::CollectionMsg))
                                .width(Length::FillPortion(1))
                                .height(Length::Fill);

                        let content = row![
                            main_content.width(Length::FillPortion(2)),
                            rule::vertical(1),
                            history_panel.width(Length::FillPortion(1)),
                            rule::vertical(1),
                            collections_panel.width(Length::FillPortion(1)),
                        ];

                        container(content)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .into()
                    } else {
                        let content = row![
                            main_content.width(Length::FillPortion(3)),
                            rule::vertical(1),
                            history_panel,
                        ];

                        container(content)
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .into()
                    }
                } else if self.show_collections {
                    let collections_panel =
                        container(self.collection_view.view().map(Message::CollectionMsg))
                            .width(Length::FillPortion(1))
                            .height(Length::Fill);

                    let content = row![
                        main_content.width(Length::FillPortion(3)),
                        rule::vertical(1),
                        collections_panel,
                    ];

                    container(content)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                } else {
                    container(main_content)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                };

                stack![content, toast_overlay].into()
            }
            View::EnvironmentManager => self.env_manager_view.view().map(Message::EnvManagerMsg),
        }
    }
}
