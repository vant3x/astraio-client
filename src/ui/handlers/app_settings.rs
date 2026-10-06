use crate::ui::app::AstraioApp;
use crate::ui::message::Message;
use crate::ui::views::app_settings_view;
use iced::Task;

pub fn handle_message(app: &mut AstraioApp, msg: app_settings_view::Message) -> Task<Message> {
    match msg {
        app_settings_view::Message::Close => {
            app.current_view = crate::ui::message::View::Main;
            Task::none()
        }
        app_settings_view::Message::ToggleTheme => {
            app.dark_mode = !app.dark_mode;
            app.app_settings_view.dark_mode = app.dark_mode;
            let theme_value = if app.dark_mode { "dark" } else { "light" };
            let _ =
                crate::persistence::database::set_app_setting(&app.db_conn, "theme", theme_value);
            Task::none()
        }
        app_settings_view::Message::ThemeSelected(theme) => {
            app.app_settings_view.highlighter_theme = theme;
            // Propagate to all request tabs
            for tab in &mut app.request_tabs {
                tab.highlighter_theme = theme;
            }
            Task::none()
        }
        app_settings_view::Message::TimeoutChanged(v) => {
            app.app_settings_view.timeout = v;
            Task::none()
        }
        app_settings_view::Message::FollowRedirectsToggled(v) => {
            app.app_settings_view.follow_redirects = v;
            Task::none()
        }
        app_settings_view::Message::MaxRedirectsChanged(v) => {
            app.app_settings_view.max_redirects = v;
            Task::none()
        }
        app_settings_view::Message::RetryCountChanged(v) => {
            app.app_settings_view.retry_count = v;
            Task::none()
        }
        app_settings_view::Message::RetryBackoffChanged(v) => {
            app.app_settings_view.retry_backoff = v;
            Task::none()
        }
        app_settings_view::Message::ProxyUrlChanged(v) => {
            app.app_settings_view.proxy_url = v;
            Task::none()
        }
        app_settings_view::Message::ProxyAuthUsernameChanged(v) => {
            app.app_settings_view.proxy_username = v;
            Task::none()
        }
        app_settings_view::Message::ProxyAuthPasswordChanged(v) => {
            app.app_settings_view.proxy_password = v;
            Task::none()
        }
        app_settings_view::Message::VerifySslToggled(v) => {
            app.app_settings_view.verify_ssl = v;
            Task::none()
        }
        app_settings_view::Message::CookieStoreToggled(v) => {
            app.app_settings_view.cookie_store = v;
            Task::none()
        }
        app_settings_view::Message::CaCertPathChanged(v) => {
            app.app_settings_view.ca_cert_path = v;
            Task::none()
        }
        app_settings_view::Message::ClientCertPathChanged(v) => {
            app.app_settings_view.client_cert_path = v;
            Task::none()
        }
        app_settings_view::Message::ClientKeyPathChanged(v) => {
            app.app_settings_view.client_key_path = v;
            Task::none()
        }
        app_settings_view::Message::UserAgentChanged(v) => {
            app.app_settings_view.user_agent = v;
            Task::none()
        }
        app_settings_view::Message::MaxBodySizeChanged(v) => {
            app.app_settings_view.max_body_size = v;
            Task::none()
        }
        app_settings_view::Message::ToggleShowAiButton(v) => {
            app.app_settings_view.show_ai_button = v;
            app.show_ai_button = v;
            let _ = crate::persistence::database::set_app_setting(
                &app.db_conn,
                "show_ai_button",
                if v { "true" } else { "false" },
            );
            Task::none()
        }
        app_settings_view::Message::OpenAiSettings => {
            app.app_settings_view.show_ai_button = app.show_ai_button;
            app.ai_view.show_settings = true;
            app.active_protocol = crate::ui::message::Protocol::Ai;
            app.current_view = crate::ui::message::View::Main;
            Task::none()
        }
        app_settings_view::Message::ClearKeychainSecrets => {
            // Re-use existing logic from app.rs
            let store = app.secret_store.clone();
            let conn = &app.db_conn;
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

            if let Ok(mut stmt) =
                conn.prepare("SELECT id FROM request_history WHERE request_data LIKE '%OAuth2%'")
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
        app_settings_view::Message::SaveSettings => {
            let config = app.app_settings_view.build_config();
            app.global_config = config.clone();

            // Propagate to all request tabs
            for tab in &mut app.request_tabs {
                tab.request_config = config.request_config.clone();
            }

            // Persist
            if let Err(e) = config.save(&app.db_conn) {
                app.toast_manager
                    .error(format!("Failed to save settings: {e}"));
            } else {
                // Persist AI button visibility
                let _ = crate::persistence::database::set_app_setting(
                    &app.db_conn,
                    "show_ai_button",
                    if app.app_settings_view.show_ai_button {
                        "true"
                    } else {
                        "false"
                    },
                );
                app.toast_manager.success("Settings saved");
                app.app_settings_view.saved = true;
            }
            Task::none()
        }
        app_settings_view::Message::ResetToDefaults => {
            let default_config = crate::http_client::config::GlobalConfig::default();
            app.app_settings_view
                .sync_from_config(&default_config, app.dark_mode);

            // Reset AI button visibility to default
            app.app_settings_view.show_ai_button = true;
            app.show_ai_button = true;
            let _ = crate::persistence::database::set_app_setting(
                &app.db_conn,
                "show_ai_button",
                "true",
            );

            // Apply to app
            app.global_config = default_config.clone();
            for tab in &mut app.request_tabs {
                tab.request_config = default_config.request_config.clone();
            }

            // Persist
            if let Err(e) = default_config.save(&app.db_conn) {
                app.toast_manager
                    .error(format!("Failed to reset settings: {e}"));
            } else {
                app.toast_manager.success("Settings reset to defaults");
            }
            Task::none()
        }
    }
}
