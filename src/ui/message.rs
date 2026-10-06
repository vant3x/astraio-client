use crate::ui::views::http_request_view;
use crate::ui::views::environment_manager;
use crate::ui::views::history_view;
use crate::ui::views::collection_view;
use crate::ui::views::websocket_view;
use crate::ui::views::graphql_view;
use crate::ui::views::mock_server_view;
use crate::ui::views::cookie_manager;
use crate::ui::views::ai_chat_view;
use crate::ui::views::app_settings_view;
use crate::ui::views::collection_runner;
use crate::protocols::websocket::WsSender;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Http,
    WebSocket,
    GraphQL,
    MockServer,
    Ai,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Http => write!(f, "HTTP"),
            Protocol::WebSocket => write!(f, "WebSocket"),
            Protocol::GraphQL => write!(f, "GraphQL"),
            Protocol::MockServer => write!(f, "Mock Server"),
            Protocol::Ai => write!(f, "AI"),
        }
    }
}

impl Protocol {
    pub const ALL: [Protocol; 4] = [
        Protocol::Http,
        Protocol::WebSocket,
        Protocol::GraphQL,
        Protocol::MockServer,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Main,
    EnvironmentManager,
    CookieManager,
    AppSettings,
}

#[derive(Debug, Clone)]
pub enum Message {
    // HTTP Request tab messages
    HttpRequestViewMsg(usize, http_request_view::Message),
    AddRequestTab,
    CloseRequestTab(usize),
    CloseActiveRequestTab,
    SelectRequestTab(usize),
    PrevRequestTab,
    NextRequestTab,
    SendActiveRequest,
    ToggleResponseSearch,

    // Environment messages
    EnvManagerMsg(environment_manager::Message),
    EnvFileLoaded(Option<Vec<(String, String)>>),
    EnvFileExported(Option<String>),
    SelectEnvironment(i32),
    SwitchView(View),
    ToggleEnvironmentManager,
    ToggleEnvInfo,

    // History messages
    HistoryMsg(history_view::Message),
    HistoryExportComplete(Option<String>),
    HistorySearchDebounced,
    ToggleHistory,

    // Collection messages
    CollectionMsg(collection_view::Message),
    ToggleCollections,

    // WebSocket messages
    WebSocketMsg(websocket_view::Message),
    WsEvent(crate::protocols::websocket::WsEvent),
    WsConnected(
        WsSender,
        Arc<Mutex<Option<mpsc::UnboundedReceiver<crate::protocols::websocket::WsEvent>>>>,
        Option<mpsc::UnboundedSender<()>>,
        Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
        Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
        Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    ),
    WsSendFromKeyboard,

    // GraphQL messages
    GraphQLMsg(graphql_view::Message),
    GraphQLOAuth2StartAuth,
    GraphQLOAuth2AuthComplete(Result<String, crate::error::AppError>, Option<String>),
    GraphQLOAuth2TokenReceived(
        Result<crate::data::oauth2::OAuth2TokenResponse, crate::error::AppError>,
    ),
    GraphQLOAuth2RefreshToken,
    GraphQLOAuth2StartDeviceAuth,
    GraphQLOAuth2DeviceAuthReceived(
        Result<crate::data::oauth2::DeviceAuthorizationResponse, crate::error::AppError>,
    ),
    GraphQLOAuth2DeviceTokenPoll(
        Result<crate::data::oauth2::DeviceTokenResponse, crate::error::AppError>,
    ),
    GraphQLOAuth2AutoPollToggle(bool),

    // OAuth2 messages (HTTP tabs)
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

    // Mock server messages
    MockServerMsg(mock_server_view::Message),
    MockServerStarted(i32, crate::protocols::mock_server::MockServerHandle, u16),
    MockServerStartError(i32, String),
    PollMockServerLogs,

    // AI messages
    AiMsg(ai_chat_view::Message),
    AiStreamReady(std::sync::Arc<tokio::sync::Mutex<Option<tokio::sync::mpsc::Receiver<String>>>>),

    // Cookie messages
    CookieManagerMsg(cookie_manager::Message),
    ToggleCookieManager,
    ClearCookies,
    ClearDomainCookies(String),
    DeleteCookie(String, String, String),
    SaveCookieEdit(String, String, String, String),
    ImportCookies,
    ImportCookiesData(Option<String>),
    ExportCookies,
    ExportCookiesComplete(Option<String>),

    // App settings messages
    AppSettingsMsg(app_settings_view::Message),

    // Collection runner messages
    CollectionRunnerMsg(collection_runner::Message),

    // HTTP streaming
    HttpStreamChunk(usize, crate::http_client::response::HttpStreamEvent),

    // Protocol/theme
    SelectProtocol(Protocol),
    ToggleTheme,

    // Keychain/secrets
    ClearKeychainSecrets,
    KeychainCleared(Result<u32, crate::error::AppError>),

    // UI state
    EscapePressed,
    ToggleSidebar,
    ShowAbout,
    Quit,
    WindowOpened(iced::window::Id),
    NoOp,
}
