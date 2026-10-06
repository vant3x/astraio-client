use crate::cookie::CookieJar;
use crate::persistence::database::{self, Environment};
use crate::ui::state::{AiStreamState, HttpStreamState, MockState, WsState};
use crate::ui::toast::ToastManager;
use crate::ui::views::collection_view::CollectionView;
use crate::ui::views::environment_manager::EnvironmentManagerView;
use crate::ui::views::history_view::HistoryView;
use crate::ui::views::websocket_view::WebSocketView;
use crate::ui::message::{Message, Protocol, View};
use crate::ui::views::http_request_view::CookieSnapshot;
use crate::ui::views::http_request_view::HttpRequestView;
use std::collections::HashMap;
use std::sync::Arc;

pub fn main() -> iced::Result {
    iced::application(AstraioApp::new, AstraioApp::update, AstraioApp::view)
        .title("Astraio Client")
        .subscription(AstraioApp::subscription)
        .theme(AstraioApp::theme)
        .font(iced_fonts::LUCIDE_FONT_BYTES)
        .run()
}

pub(crate) struct AstraioApp {
    pub(crate) request_tabs: Vec<HttpRequestView>,
    pub(crate) active_request_tab_index: usize,
    pub(crate) http_client: Arc<reqwest::Client>,
    pub(crate) custom_clients: HashMap<String, (Arc<reqwest::Client>, std::time::Instant)>,
    pub(crate) cookie_jar: Arc<std::sync::Mutex<CookieJar>>,
    pub(crate) db_conn: rusqlite::Connection,
    pub(crate) environments: Vec<Environment>,
    pub(crate) active_environment: Option<Environment>,
    pub(crate) env_manager_view: EnvironmentManagerView,
    pub(crate) history_view: HistoryView,
    pub(crate) collection_view: CollectionView,
    pub(crate) websocket_view: WebSocketView,
    pub(crate) graphql_view: crate::ui::views::graphql_view::GraphQLView,
    pub(crate) mock: MockState,
    pub(crate) active_protocol: Protocol,
    pub(crate) current_view: View,
    pub(crate) show_history: bool,
    pub(crate) show_collections: bool,
    pub(crate) show_env_info: bool,
    pub(crate) ws: WsState,
    pub(crate) http_stream: HttpStreamState,
    pub(crate) toast_manager: ToastManager,
    pub(crate) dark_mode: bool,
    pub(crate) secret_store: crate::services::secret_store::SecretStore,
    pub(crate) global_config: crate::http_client::config::GlobalConfig,
    pub(crate) main_window_id: Option<iced::window::Id>,
    pub(crate) cookie_manager_view: crate::ui::views::cookie_manager::CookieManagerView,
    pub(crate) show_collection_runner: bool,
    pub(crate) collection_runner_state:
        Option<crate::ui::views::collection_runner::CollectionRunnerState>,
    pub(crate) ai_view: crate::ui::views::ai_chat_view::AiChatView,
    pub(crate) ai_service: std::sync::Arc<tokio::sync::Mutex<crate::ai::service::AiService>>,
    pub(crate) ai_stream: AiStreamState,
    pub(crate) app_settings_view: crate::ui::views::app_settings_view::AppSettingsView,
    pub(crate) show_ai_button: bool,
    pub(crate) cookie_cleanup_counter: u32,
}

impl Drop for AstraioApp {
    fn drop(&mut self) {
        self.cleanup();
    }
}

impl AstraioApp {
    fn new() -> (Self, iced::Task<Message>) {
        let (db_conn, environments) = match database::init() {
            Ok(conn) => {
                let envs =
                    crate::services::environment_service::get_all(&conn).unwrap_or_else(|e| {
                        log::error!("Failed to load environments: {e}");
                        Vec::new()
                    });
                (conn, envs)
            }
            Err(e) => {
                log::error!("Failed to initialize database: {e}");
                let conn = rusqlite::Connection::open_in_memory()
                    .expect("In-memory DB should always work");
                if let Err(schema_err) = database::init_schema(&conn) {
                    log::error!("Failed to init in-memory schema: {schema_err}");
                }
                (conn, Vec::new())
            }
        };

        let history =
            crate::services::history_service::get_all(&db_conn, 200).unwrap_or_else(|e| {
                log::error!("Failed to load history: {e}");
                Vec::new()
            });
        let collections =
            crate::services::collection_service::get_all(&db_conn).unwrap_or_else(|e| {
                log::error!("Failed to load collections: {e}");
                Vec::new()
            });

        let mut cv = CollectionView::new();
        cv.sync_collections(&collections);

        let mock_servers =
            crate::services::mock_server_service::get_all(&db_conn).unwrap_or_else(|e| {
                log::warn!("Failed to load mock servers: {e}");
                Vec::new()
            });

        let secret_store = crate::services::secret_store::SecretStore::new();
        match crate::services::secret_store::migrate_plaintext_tokens_to_keyring(
            &secret_store,
            &db_conn,
        ) {
            Ok(0) => {}
            Ok(n) => log::info!("Migrated {n} plaintext tokens to OS keyring"),
            Err(e) => log::warn!("Keyring migration skipped: {e}"),
        }

        let dark_mode = crate::persistence::database::get_app_setting(&db_conn, "theme")
            .is_none_or(|v| v != "light");

        let global_config = crate::http_client::config::GlobalConfig::load(&db_conn);

        let sessions = crate::persistence::database::load_sessions(&db_conn).unwrap_or_else(|e| {
            log::warn!("Failed to load sessions: {e}");
            Vec::new()
        });

        let ai_provider_configs =
            crate::persistence::database::get_all_ai_providers(&db_conn).unwrap_or_default();
        let ai_service_init = {
            let mut svc = crate::ai::service::AiService::new();
            for config in &ai_provider_configs {
                let secret_key = format!("{}_{}", config.provider, config.name);
                let api_key = secret_store
                    .get_secret("ai", &secret_key, "api_key")
                    .ok()
                    .flatten();
                svc.register_provider(config.clone(), api_key);
            }
            std::sync::Arc::new(tokio::sync::Mutex::new(svc))
        };
        let ai_view_init = {
            let mut view = crate::ui::views::ai_chat_view::AiChatView::new();
            if !ai_provider_configs.is_empty() {
                view.active_provider_index = ai_provider_configs
                    .iter()
                    .position(|p| p.is_default)
                    .or(Some(0));
            }
            view.providers = ai_provider_configs;
            view
        };

        let default_tab = HttpRequestView {
            request_config: global_config.request_config.clone(),
            sessions: sessions.clone(),
            ..HttpRequestView::default()
        };

        let mut app_settings_view = crate::ui::views::app_settings_view::AppSettingsView::new();
        app_settings_view.sync_from_config(&global_config, dark_mode);

        let show_ai_button = crate::persistence::database::get_app_setting(&db_conn, "show_ai_button")
            .is_none_or(|v| v != "false");
        app_settings_view.show_ai_button = show_ai_button;

        let app = Self {
            request_tabs: vec![default_tab],
            active_request_tab_index: 0,
            http_client: Arc::new(
                reqwest::Client::builder()
                    .cookie_store(true)
                    .build()
                    .unwrap_or_else(|_| reqwest::Client::new()),
            ),
            custom_clients: HashMap::new(),
            cookie_jar: Arc::new(std::sync::Mutex::new(
                crate::persistence::database::load_cookies(&db_conn).unwrap_or_else(|e| {
                    log::warn!("Failed to load cookies from SQLite: {e}");
                    CookieJar::new()
                }),
            )),
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
            graphql_view: crate::ui::views::graphql_view::GraphQLView::default(),
            mock: {
                let mut mv = crate::ui::views::mock_server_view::MockServerView::default();
                mv.sync_servers(&mock_servers);
                MockState::new(mv)
            },
            active_protocol: Protocol::Http,
            current_view: View::Main,
            show_history: false,
            show_collections: false,
            show_env_info: false,
            ws: WsState::new(),
            http_stream: HttpStreamState::new(),
            toast_manager: ToastManager::new(),
            dark_mode,
            secret_store,
            global_config,
            main_window_id: None,
            cookie_manager_view: crate::ui::views::cookie_manager::CookieManagerView::default(),
            show_collection_runner: false,
            collection_runner_state: None,
            ai_view: ai_view_init,
            ai_service: ai_service_init,
            ai_stream: AiStreamState::new(),
            app_settings_view,
            show_ai_button,
            cookie_cleanup_counter: 0,
        };
        (app, iced::Task::none())
    }

    fn cleanup(&mut self) {
        self.ws.shutdown();
        if let Ok(jar) = self.cookie_jar.lock() {
            if let Err(e) = crate::persistence::database::save_cookies(&self.db_conn, &jar) {
                log::warn!("Failed to persist cookies on shutdown: {e}");
            }
        }
        for (id, handle) in self.mock.handles.drain() {
            crate::protocols::mock_server::stop_mock_server(handle);
            log::info!("[Mock] Stopped mock server id={id}");
        }
        log::info!("Astraio cleanup complete");
    }

    pub(crate) fn sync_cookie_data_to_tabs(&mut self) {
        let jar = match self.cookie_jar.lock() {
            Ok(jar) => jar,
            Err(e) => {
                log::error!("Failed to acquire cookie_jar lock for sync: {e}");
                return;
            }
        };

        let domains: Vec<(String, usize)> = jar
            .domains()
            .into_iter()
            .map(|(d, c)| (d.to_string(), c))
            .collect();
        let total = jar.total_count();
        let domain_count = jar.domain_count();

        let active_idx = self.active_request_tab_index;

        let active_cookies: Option<Vec<CookieSnapshot>> = if active_idx < self.request_tabs.len() {
            let mut all_cookies = Vec::with_capacity(total);
            for (d, _) in &domains {
                for c in jar.cookies_for_domain(d) {
                    all_cookies.push(CookieSnapshot {
                        name: c.name.clone(),
                        value: c.value.clone(),
                        domain: c.domain.clone(),
                        path: c.path.clone(),
                        secure: c.secure,
                        http_only: c.http_only,
                        same_site: c.same_site.to_string(),
                        expires: c.expires.clone(),
                    });
                }
            }
            Some(all_cookies)
        } else {
            None
        };
        drop(jar);

        for (i, tab) in self.request_tabs.iter_mut().enumerate() {
            tab.cookie_count = total;
            tab.cookie_domain_count = domain_count;
            if i == active_idx {
                if let Some(ref cookies) = active_cookies {
                    tab.cookie_domains = domains.clone();
                    tab.cookie_domain_cookies = cookies.clone();
                }
            } else {
                tab.cookie_domains.clear();
                tab.cookie_domain_cookies.clear();
            }
        }
    }

    pub(crate) fn send_ws_message(websocket_view: &mut crate::ui::views::websocket_view::WebSocketView) {
        use crate::protocols::websocket::WsStatus;
        if let Some(sender) = &websocket_view.ws_sender {
            let input = websocket_view.input.clone();
            if !input.is_empty() && matches!(websocket_view.status, WsStatus::Connected) {
                let bytes = input.len() as u64;
                let _ = sender.send(&input);
                websocket_view.stats.messages_sent += 1;
                websocket_view.stats.bytes_sent += bytes;
                websocket_view.last_sent_message = input.clone();
                websocket_view.add_message(crate::protocols::websocket::WsMessage::outgoing(input));
                websocket_view.input.clear();
            }
        }
    }
}
