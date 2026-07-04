use crate::http_client::client;
use crate::ui::app::{AstraNovaApp, Message};
use crate::ui::views::http_request_view;
use iced::Task;
use std::sync::Arc;

pub fn handle_http_request_msg(
    app: &mut AstraNovaApp,
    index: usize,
    msg: http_request_view::Message,
) -> Task<Message> {
    let view = match app.request_tabs.get_mut(index) {
        Some(v) => v,
        None => return Task::none(),
    };

    match msg {
        http_request_view::Message::SendRequest => {
            let mut temp_view = view.clone();
            if let Some(env) = &app.active_environment {
                temp_view.apply_environment(env);
            }
            let request = temp_view.build_request();
            view.pending_request_data = serde_json::to_string(&request).ok();
            view.update(http_request_view::Message::SetLoading);

            let http_client =
                if request.config.proxy_url.is_some() || !request.config.verify_ssl {
                    let cache_key = format!(
                        "{}|{}",
                        request.config.proxy_url.as_deref().unwrap_or(""),
                        request.config.verify_ssl
                    );
                    if let Some(cached) = app.custom_clients.get(&cache_key) {
                        Arc::clone(cached)
                    } else {
                        match client::build_client(&request.config) {
                            Ok(c) => {
                                let c = Arc::new(c);
                                app.custom_clients.insert(cache_key, Arc::clone(&c));
                                c
                            }
                            Err(e) => {
                                log::error!("Failed to build custom client: {}", e);
                                Arc::clone(&app.http_client)
                            }
                        }
                    }
                } else {
                    Arc::clone(&app.http_client)
                };

            Task::perform(
                async move { client::send_request(&http_client, request).await },
                move |result| {
                    Message::HttpRequestViewMsg(
                        index,
                        http_request_view::Message::ResponseReceived(result),
                    )
                },
            )
        }
        http_request_view::Message::ResponseReceived(ref result) => {
            let view = app.request_tabs.get_mut(index).unwrap();
            match result {
                Ok(response) => {
                    let request_data = view.pending_request_data.take();
                    let response_data = serde_json::to_string(response).ok();
                    let _ = crate::services::history_service::save_raw(
                        &app.db_conn,
                        &response.method.to_string(),
                        &response.url,
                        Some(response.status),
                        Some(response.duration.as_millis() as u64),
                        request_data.as_deref(),
                        response_data.as_deref(),
                    );
                    crate::services::history_service::trim(
                        &app.db_conn,
                        crate::persistence::database::DEFAULT_HISTORY_LIMIT,
                    );
                    app.history_view.entries =
                        crate::services::history_service::get_all(&app.db_conn, 200);

                    if response.status >= 400 {
                        app.toast_manager
                            .warning(format!("{} {}", response.status, response.url));
                    } else {
                        app.toast_manager.success(format!(
                            "{} {} - {}ms",
                            response.status,
                            response.url,
                            response.duration.as_millis()
                        ));
                    }
                }
                Err(e) => {
                    app.toast_manager
                        .error(format!("Request failed: {}", e));
                }
            }
            view.update(msg);
            Task::none()
        }
        http_request_view::Message::MultipartBrowseFile(entry_id) => {
            let tab_index = index;
            Task::perform(
                async {
                    let file = rfd::AsyncFileDialog::new().pick_file().await;
                    file.map(|f| f.path().to_string_lossy().to_string())
                },
                move |path| {
                    Message::HttpRequestViewMsg(
                        tab_index,
                        http_request_view::Message::MultipartFilePicked(entry_id, path),
                    )
                },
            )
        }
        http_request_view::Message::OAuth2StartAuth => {
            Task::perform(async {}, move |_| Message::OAuth2StartAuth(index))
        }
        http_request_view::Message::OAuth2RefreshToken => {
            Task::perform(async {}, move |_| Message::OAuth2RefreshToken(index))
        }
        http_request_view::Message::OAuth2StartDeviceAuth => {
            Task::perform(async {}, move |_| Message::OAuth2StartDeviceAuth(index))
        }
        http_request_view::Message::OAuth2AutoPollToggle(enabled) => {
            Task::perform(async {}, move |_| Message::OAuth2AutoPollToggle(index, enabled))
        }
        other => {
            if let Some(view) = app.request_tabs.get_mut(index) {
                view.update(other);
            }
            Task::none()
        }
    }
}
