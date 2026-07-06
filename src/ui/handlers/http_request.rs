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
            let request = match temp_view.build_request() {
                Ok(r) => r,
                Err(e) => {
                    app.toast_manager
                        .error(format!("Failed to build request: {}", e));
                    return Task::none();
                }
            };
            view.pending_request_data = serde_json::to_string(&request).ok();
            view.update(http_request_view::Message::SetLoading);

            let http_client = if request.config.proxy_url.is_some() || !request.config.verify_ssl {
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
            let Some(view) = app.request_tabs.get_mut(index) else {
                return Task::none();
            };
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
                    app.toast_manager.error(format!("Request failed: {}", e));
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
            Task::perform(async {}, move |_| {
                Message::OAuth2AutoPollToggle(index, enabled)
            })
        }
        http_request_view::Message::DownloadResponse => {
            let view = app.request_tabs.get(index).cloned();
            Task::perform(
                async move {
                    let view = match view {
                        Some(v) => v,
                        None => return Err("No tab".to_string()),
                    };
                    let response = match &view.last_response {
                        Some(r) => r.clone(),
                        None => return Err("No response".to_string()),
                    };
                    let content_type = response
                        .headers
                        .iter()
                        .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
                        .map(|(_, v)| v.clone())
                        .unwrap_or_else(|| "application/octet-stream".to_string());

                    let ext = if content_type.contains("image/png") {
                        "png"
                    } else if content_type.contains("image/jpeg")
                        || content_type.contains("image/jpg")
                    {
                        "jpg"
                    } else if content_type.contains("image/gif") {
                        "gif"
                    } else if content_type.contains("image/svg") {
                        "svg"
                    } else if content_type.contains("image/webp") {
                        "webp"
                    } else if content_type.contains("application/pdf") {
                        "pdf"
                    } else if content_type.contains("application/zip") {
                        "zip"
                    } else if content_type.contains("application/gzip")
                        || content_type.contains("application/x-gzip")
                    {
                        "gz"
                    } else if content_type.contains("application/json") {
                        "json"
                    } else if content_type.contains("application/xml")
                        || content_type.contains("text/xml")
                    {
                        "xml"
                    } else if content_type.contains("text/html") {
                        "html"
                    } else {
                        "bin"
                    };

                    let suggested_name = format!("response.{}", ext);

                    let save_dialog = rfd::AsyncFileDialog::new()
                        .set_file_name(&suggested_name)
                        .save_file()
                        .await;

                    let file_handle = match save_dialog {
                        Some(h) => h,
                        None => return Err("Cancelled".to_string()),
                    };

                    let bytes = if response.body_encoding
                        == crate::http_client::response::BodyEncoding::Base64
                    {
                        use base64::Engine;
                        base64::engine::general_purpose::STANDARD
                            .decode(&response.body)
                            .unwrap_or_default()
                    } else {
                        response.body.into_bytes()
                    };

                    std::fs::write(file_handle.path(), &bytes)
                        .map_err(|e| format!("Write error: {}", e))?;

                    Ok(file_handle.path().to_string_lossy().to_string())
                },
                move |result| {
                    Message::HttpRequestViewMsg(
                        index,
                        http_request_view::Message::ResponseFileSaved(result),
                    )
                },
            )
        }
        http_request_view::Message::ResponseFileSaved(result) => {
            match result {
                Ok(path) => {
                    app.toast_manager.success(format!("Saved: {}", path));
                }
                Err(e) => {
                    if e != "Cancelled" {
                        app.toast_manager.error(e);
                    }
                }
            }
            Task::none()
        }
        other => {
            if let Some(view) = app.request_tabs.get_mut(index) {
                view.update(other);
            }
            Task::none()
        }
    }
}
