use crate::ui::app::AstraioApp;
use crate::ui::message::{Message, Protocol, View};
use crate::ui::views::http_request_view;
use crate::ui::views::graphql_view;
use iced::Task;

impl AstraioApp {
    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        self.toast_manager.clean_expired();
        match message {
            Message::HttpRequestViewMsg(index, msg) => {
                super::handlers::http_request::handle_http_request_msg(self, index, msg)
            }
            Message::HttpStreamChunk(tab_index, event) => {
                use crate::http_client::response::HttpStreamEvent;
                if let Some(view) = self.request_tabs.get_mut(tab_index) {
                    view.update(http_request_view::Message::StreamEvent(
                        tab_index,
                        event.clone(),
                    ));

                    if matches!(
                        event,
                        HttpStreamEvent::StreamComplete { .. } | HttpStreamEvent::StreamError(_)
                    ) {
                        self.http_stream.receivers.remove(&tab_index);
                    }
                }
                Task::none()
            }
            Message::AddRequestTab => {
                let mut new_view = http_request_view::HttpRequestView {
                    request_config: self.global_config.request_config.clone(),
                    sessions: crate::persistence::database::load_sessions(&self.db_conn)
                        .unwrap_or_default(),
                    ..http_request_view::HttpRequestView::default()
                };
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
                    self.http_stream.receivers.remove(&index);
                    if self.active_request_tab_index >= self.request_tabs.len() {
                        self.active_request_tab_index = self.request_tabs.len() - 1;
                    }
                    self.sync_cookie_data_to_tabs();
                }
                Task::none()
            }
            Message::CloseActiveRequestTab => {
                if self.request_tabs.len() > 1 {
                    let index = self.active_request_tab_index;
                    self.request_tabs.remove(index);
                    self.http_stream.receivers.remove(&index);
                    if self.active_request_tab_index >= self.request_tabs.len() {
                        self.active_request_tab_index = self.request_tabs.len() - 1;
                    }
                    self.sync_cookie_data_to_tabs();
                }
                Task::none()
            }
            Message::NoOp => Task::none(),
            Message::SelectRequestTab(index) => {
                self.active_request_tab_index = index;
                self.sync_cookie_data_to_tabs();
                Task::none()
            }
            Message::PrevRequestTab => {
                if !self.request_tabs.is_empty() {
                    self.active_request_tab_index =
                        (self.active_request_tab_index + self.request_tabs.len() - 1)
                            % self.request_tabs.len();
                    self.sync_cookie_data_to_tabs();
                }
                Task::none()
            }
            Message::NextRequestTab => {
                if !self.request_tabs.is_empty() {
                    self.active_request_tab_index =
                        (self.active_request_tab_index + 1) % self.request_tabs.len();
                    self.sync_cookie_data_to_tabs();
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
                    View::CookieManager => View::CookieManager,
                    View::AppSettings => View::AppSettings,
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
                            log::error!("Failed to refresh collections: {e}");
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
                let theme_value = if self.dark_mode { "dark" } else { "light" };
                let _ = crate::persistence::database::set_app_setting(
                    &self.db_conn,
                    "theme",
                    theme_value,
                );
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
            Message::HistorySearchDebounced => {
                super::handlers::history::refresh_history_entries(self);
                Task::none()
            }
            Message::SelectProtocol(protocol) => {
                self.active_protocol = protocol;
                Task::none()
            }
            Message::WsEvent(event) => super::handlers::websocket::handle_ws_event(self, event),
            Message::WebSocketMsg(msg) => super::handlers::websocket::handle_message(self, msg),
            Message::GraphQLMsg(msg) => super::handlers::graphql::handle_message(self, msg),
            Message::MockServerMsg(msg) => super::handlers::mock_server::handle_message(self, msg),
            Message::AiMsg(msg) => super::handlers::ai::handle_message(self, msg),
            Message::AiStreamReady(rx) => {
                self.ai_stream.set_receiver(rx);
                Task::none()
            }
            Message::AppSettingsMsg(msg) => {
                super::handlers::app_settings::handle_message(self, msg)
            }
            Message::MockServerStarted(id, handle, actual_port) => {
                self.mock.handles.insert(id, handle);
                self.mock.view.statuses.insert(
                    id,
                    crate::protocols::mock_server::MockServerStatus::Running { actual_port },
                );
                self.toast_manager
                    .success(format!("Mock server running on port {actual_port}"));
                Task::none()
            }
            Message::MockServerStartError(id, error) => {
                self.mock.view.statuses.insert(
                    id,
                    crate::protocols::mock_server::MockServerStatus::Error(error.clone()),
                );
                self.toast_manager
                    .error(format!("Mock server error: {error}"));
                Task::none()
            }
            Message::PollMockServerLogs => {
                for handle in self.mock.handles.values() {
                    if let Ok(mut rx) = handle.log_rx.try_lock() {
                        while let Ok(log) = rx.try_recv() {
                            self.mock.view.logs.push(log);
                        }
                    }
                }

                self.cookie_cleanup_counter += 1;
                if self.cookie_cleanup_counter >= 300 {
                    self.cookie_cleanup_counter = 0;
                    if let Ok(mut jar) = self.cookie_jar.lock() {
                        jar.remove_expired();
                        if let Err(e) =
                            crate::persistence::database::save_cookies(&self.db_conn, &jar)
                        {
                            log::warn!("Failed to persist cookies after cleanup: {e}");
                        }
                    }
                }

                Task::none()
            }
            Message::GraphQLOAuth2StartAuth => {
                super::handlers::oauth2::handle_graphql_start_auth(self)
            }
            Message::GraphQLOAuth2AuthComplete(result, pkce_verifier) => {
                super::handlers::oauth2::handle_graphql_auth_complete(self, result, pkce_verifier)
            }
            Message::GraphQLOAuth2TokenReceived(result) => {
                super::handlers::oauth2::handle_graphql_token_received(self, result)
            }
            Message::GraphQLOAuth2RefreshToken => {
                super::handlers::oauth2::handle_graphql_refresh_token(self)
            }
            Message::GraphQLOAuth2StartDeviceAuth => {
                super::handlers::oauth2::handle_graphql_start_device_auth(self)
            }
            Message::GraphQLOAuth2DeviceAuthReceived(result) => {
                super::handlers::oauth2::handle_graphql_device_auth_received(self, result)
            }
            Message::GraphQLOAuth2DeviceTokenPoll(result) => {
                super::handlers::oauth2::handle_graphql_device_token_poll(self, result)
            }
            Message::GraphQLOAuth2AutoPollToggle(enabled) => {
                super::handlers::oauth2::handle_graphql_auto_poll_toggle(self, enabled)
            }
            Message::WsConnected(
                sender,
                receiver_arc,
                shutdown_tx,
                write_handle,
                read_handle,
                ping_handle,
            ) => {
                super::handlers::websocket::handle_ws_connected(
                    self,
                    sender,
                    receiver_arc,
                    shutdown_tx,
                    write_handle,
                    read_handle,
                    ping_handle,
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
                Self::send_ws_message(&mut self.websocket_view);
                Task::none()
            }
            Message::SendActiveRequest => {
                match self.active_protocol {
                    Protocol::WebSocket => {
                        Self::send_ws_message(&mut self.websocket_view);
                    }
                    Protocol::GraphQL => {
                        return super::handlers::graphql::handle_message(
                            self,
                            graphql_view::Message::SendRequest,
                        );
                    }
                    Protocol::Http => {
                        return super::handlers::http_request::handle_http_request_msg(
                            self,
                            self.active_request_tab_index,
                            http_request_view::Message::SendRequest,
                        );
                    }
                    Protocol::MockServer => {}
                    Protocol::Ai => {
                        return super::handlers::ai::handle_message(
                            self,
                            crate::ui::views::ai_chat_view::Message::SendMessage,
                        );
                    }
                }
                Task::none()
            }
            Message::EscapePressed => {
                if self.show_env_info {
                    self.show_env_info = false;
                } else if let Some(view) = self.request_tabs.get(self.active_request_tab_index) {
                    if view.show_response_search {
                        return super::handlers::http_request::handle_http_request_msg(
                            self,
                            self.active_request_tab_index,
                            http_request_view::Message::ToggleResponseSearch,
                        );
                    } else if view.show_snippets {
                        return super::handlers::http_request::handle_http_request_msg(
                            self,
                            self.active_request_tab_index,
                            http_request_view::Message::ShowSnippets,
                        );
                    }
                }
                Task::none()
            }
            Message::ClearKeychainSecrets => {
                let store = self.secret_store.clone();
                let conn = &self.db_conn;
                let mut identifiers = Vec::new();

                if let Ok(mut stmt) = conn.prepare(
                    "SELECT id, collection_id FROM collection_requests WHERE auth_type = 'oauth2'",
                ) {
                    if let Ok(rows) =
                        stmt.query_map([], |row| Ok((row.get::<_, i32>(0)?, row.get::<_, i32>(1)?)))
                    {
                        for row in rows.flatten() {
                            identifiers.push(format!("col_{}_{}", row.1, row.0));
                        }
                    }
                }

                if let Ok(mut stmt) = conn
                    .prepare("SELECT id FROM request_history WHERE request_data LIKE '%OAuth2%'")
                {
                    if let Ok(rows) = stmt.query_map([], |row| row.get::<_, i32>(0)) {
                        for row in rows.flatten() {
                            identifiers.push(format!("hist_{row}"));
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
                        self.toast_manager
                            .success(format!("Cleared {count} keychain entries"));
                    }
                    Err(e) => {
                        self.toast_manager
                            .error(format!("Failed to clear keychain: {e}"));
                    }
                }
                Task::none()
            }
            Message::ClearCookies => {
                if let Ok(mut jar) = self.cookie_jar.lock() {
                    jar.clear();
                } else {
                    log::error!("Failed to acquire cookie_jar lock for ClearCookies");
                }
                if let Err(e) = crate::persistence::database::clear_cookies_db(&self.db_conn) {
                    log::warn!("Failed to clear cookies from DB: {e}");
                }
                for tab in &mut self.request_tabs {
                    tab.cookie_count = 0;
                    tab.cookie_domain_count = 0;
                    tab.cookie_domains.clear();
                    tab.cookie_domain_cookies.clear();
                }
                self.toast_manager.success("Cookies cleared".to_string());
                Task::none()
            }
            Message::ClearDomainCookies(domain) => {
                if let Ok(mut jar) = self.cookie_jar.lock() {
                    jar.clear_domain(&domain);
                } else {
                    log::error!("Failed to acquire cookie_jar lock for ClearDomainCookies");
                }
                if let Err(e) =
                    crate::persistence::database::clear_domain_cookies_db(&self.db_conn, &domain)
                {
                    log::warn!("Failed to clear domain cookies from DB: {e}");
                }
                self.sync_cookie_data_to_tabs();
                self.toast_manager
                    .success(format!("Cookies for {domain} cleared"));
                Task::none()
            }
            Message::DeleteCookie(domain, name, path) => {
                if let Ok(mut jar) = self.cookie_jar.lock() {
                    jar.remove_cookie(&domain, &name, &path);
                } else {
                    log::error!("Failed to acquire cookie_jar lock for DeleteCookie");
                }
                if let Err(e) = crate::persistence::database::delete_cookie_db(
                    &self.db_conn,
                    &domain,
                    &name,
                    &path,
                ) {
                    log::warn!("Failed to delete cookie from DB: {e}");
                }
                self.sync_cookie_data_to_tabs();
                Task::none()
            }
            Message::SaveCookieEdit(domain, name, path, new_value) => {
                if let Ok(mut jar) = self.cookie_jar.lock() {
                    if let Some(cookies) = jar.cookies_for_domain_mut(&domain) {
                        for c in cookies.iter_mut() {
                            if c.name == name && c.path == path {
                                c.value = new_value.clone();
                                break;
                            }
                        }
                    }
                } else {
                    log::error!("Failed to acquire cookie_jar lock for SaveCookieEdit");
                }
                if let Err(e) = crate::persistence::database::update_cookie_value_db(
                    &self.db_conn,
                    &domain,
                    &name,
                    &path,
                    &new_value,
                ) {
                    log::warn!("Failed to update cookie in DB: {e}");
                }
                self.sync_cookie_data_to_tabs();
                Task::none()
            }
            Message::ImportCookies => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Cookie files", &["txt", "json", "cookie", "cookies"])
                        .pick_file()
                        .await
                        .map(|f| f.path().to_path_buf())
                        .and_then(|p| std::fs::read_to_string(p).ok())
                },
                Message::ImportCookiesData,
            ),
            Message::ImportCookiesData(content) => {
                if let Some(content) = content {
                    let new_jar = if let Ok(jar) = crate::cookie::CookieJar::from_json(&content) {
                        jar
                    } else if let Ok(jar) = crate::cookie::CookieJar::from_netscape(&content) {
                        jar
                    } else {
                        self.toast_manager
                            .error("Failed to parse cookie file".to_string());
                        return Task::none();
                    };
                    {
                        if let Ok(mut jar) = self.cookie_jar.lock() {
                            for cookie in new_jar.all_cookies() {
                                jar.insert(cookie.clone());
                            }
                        }
                    }
                    if let Ok(jar) = self.cookie_jar.lock() {
                        if let Err(e) =
                            crate::persistence::database::save_cookies(&self.db_conn, &jar)
                        {
                            log::warn!("Failed to persist imported cookies: {e}");
                        }
                    } else {
                        log::error!(
                            "Failed to acquire cookie_jar lock for persisting imported cookies"
                        );
                    }
                    self.sync_cookie_data_to_tabs();
                    self.toast_manager
                        .success("Cookies imported successfully".to_string());
                }
                Task::none()
            }
            Message::ExportCookies => {
                let content = match self.cookie_jar.lock() {
                    Ok(jar) => jar.to_netscape(),
                    Err(e) => {
                        log::error!("Failed to acquire cookie_jar lock for export: {e}");
                        return Task::none();
                    }
                };
                Task::perform(
                    async move {
                        rfd::AsyncFileDialog::new()
                            .add_filter("Cookie files", &["txt"])
                            .set_file_name("cookies.txt")
                            .save_file()
                            .await
                            .and_then(|f| {
                                std::fs::write(f.path(), &content).ok()?;
                                Some(())
                            })
                    },
                    |result| {
                        Message::ExportCookiesComplete(result.map(|()| "Exported".to_string()))
                    },
                )
            }
            Message::ExportCookiesComplete(result) => {
                if let Some(msg) = result {
                    self.toast_manager.success(msg);
                }
                Task::none()
            }
            Message::ToggleSidebar => {
                self.show_collections = !self.show_collections;
                Task::none()
            }
            Message::ShowAbout => {
                self.toast_manager.info("Astraio Client v0.6.0");
                Task::none()
            }
            Message::Quit => {
                if let Some(id) = self.main_window_id {
                    iced::window::close(id)
                } else {
                    Task::none()
                }
            }
            Message::WindowOpened(id) => {
                if self.main_window_id.is_none() {
                    self.main_window_id = Some(id);
                }
                #[cfg(target_os = "macos")]
                {
                    crate::ui::menu::attach_macos();
                }
                Task::none()
            }
            Message::CookieManagerMsg(msg) => {
                use crate::ui::views::cookie_manager::CookieManagerAction;
                if let Some(action) = self.cookie_manager_view.update(msg) {
                    match action {
                        CookieManagerAction::DeleteCookie(domain, name, path) => {
                            if let Ok(mut jar) = self.cookie_jar.lock() {
                                jar.remove_cookie(&domain, &name, &path);
                            }
                            if let Err(e) = crate::persistence::database::delete_cookie_db(
                                &self.db_conn,
                                &domain,
                                &name,
                                &path,
                            ) {
                                log::warn!("Failed to delete cookie from DB: {e}");
                            }
                            self.sync_cookie_data_to_tabs();
                            self.toast_manager.success("Cookie deleted");
                        }
                        CookieManagerAction::ClearDomain(domain) => {
                            if let Ok(mut jar) = self.cookie_jar.lock() {
                                jar.clear_domain(&domain);
                            }
                            if let Err(e) = crate::persistence::database::clear_domain_cookies_db(
                                &self.db_conn,
                                &domain,
                            ) {
                                log::warn!("Failed to clear domain cookies from DB: {e}");
                            }
                            self.sync_cookie_data_to_tabs();
                            self.toast_manager
                                .success(format!("Cookies for {domain} cleared"));
                        }
                        CookieManagerAction::ClearAll => {
                            if let Ok(mut jar) = self.cookie_jar.lock() {
                                jar.clear();
                            }
                            if let Err(e) =
                                crate::persistence::database::clear_cookies_db(&self.db_conn)
                            {
                                log::warn!("Failed to clear cookies from DB: {e}");
                            }
                            for tab in &mut self.request_tabs {
                                tab.cookie_count = 0;
                                tab.cookie_domain_count = 0;
                                tab.cookie_domains.clear();
                                tab.cookie_domain_cookies.clear();
                            }
                            self.toast_manager.success("All cookies cleared");
                        }
                        CookieManagerAction::SaveEdit(domain, name, path, new_value) => {
                            if let Ok(mut jar) = self.cookie_jar.lock() {
                                if let Some(cookies) = jar.cookies_for_domain_mut(&domain) {
                                    for c in cookies.iter_mut() {
                                        if c.name == name && c.path == path {
                                            c.value = new_value.clone();
                                            break;
                                        }
                                    }
                                }
                            }
                            if let Err(e) = crate::persistence::database::update_cookie_value_db(
                                &self.db_conn,
                                &domain,
                                &name,
                                &path,
                                &new_value,
                            ) {
                                log::warn!("Failed to update cookie in DB: {e}");
                            }
                            self.sync_cookie_data_to_tabs();
                            self.toast_manager.success("Cookie updated");
                        }
                        CookieManagerAction::ImportCookies => {
                            return Task::perform(
                                async {
                                    rfd::AsyncFileDialog::new()
                                        .add_filter(
                                            "Cookie files",
                                            &["txt", "json", "cookie", "cookies"],
                                        )
                                        .pick_file()
                                        .await
                                        .map(|f| f.path().to_path_buf())
                                        .and_then(|p| std::fs::read_to_string(p).ok())
                                },
                                Message::ImportCookiesData,
                            );
                        }
                        CookieManagerAction::ExportCookies => {
                            let content = match self.cookie_jar.lock() {
                                Ok(jar) => jar.to_netscape(),
                                Err(e) => {
                                    log::error!(
                                        "Failed to acquire cookie_jar lock for export: {e}"
                                    );
                                    return Task::none();
                                }
                            };
                            return Task::perform(
                                async move {
                                    rfd::AsyncFileDialog::new()
                                        .add_filter("Cookie files", &["txt"])
                                        .set_file_name("cookies.txt")
                                        .save_file()
                                        .await
                                        .and_then(|f| {
                                            std::fs::write(f.path(), &content).ok()?;
                                            Some(())
                                        })
                                },
                                |result| {
                                    Message::ExportCookiesComplete(
                                        result.map(|()| "Exported".to_string()),
                                    )
                                },
                            );
                        }
                    }
                    if let Ok(jar) = self.cookie_jar.lock() {
                        self.cookie_manager_view.sync_from_jar(&jar);
                    }
                }
                Task::none()
            }
            Message::ToggleCookieManager => {
                self.current_view = match self.current_view {
                    View::CookieManager => View::Main,
                    View::Main => View::CookieManager,
                    View::EnvironmentManager => View::CookieManager,
                    View::AppSettings => View::CookieManager,
                };
                if self.current_view == View::CookieManager {
                    if let Ok(jar) = self.cookie_jar.lock() {
                        self.cookie_manager_view.sync_from_jar(&jar);
                    }
                }
                Task::none()
            }
            Message::CollectionRunnerMsg(msg) => {
                super::handlers::collection_runner::handle_message(self, msg)
            }
        }
    }
}
