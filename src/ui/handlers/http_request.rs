use crate::http_client::client;
use crate::protocols::scripts::{ScriptContext, ScriptEngine};
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

            // Validate URL before sending
            let url = temp_view.url_input.trim();
            if url.is_empty() {
                app.toast_manager.error("URL is required".to_string());
                return Task::none();
            }
            if reqwest::Url::parse(url).is_err() {
                app.toast_manager.error(format!("Invalid URL: {}", url));
                return Task::none();
            }

            // Auto-detect JSON body if content type is not explicitly set
            let body_text = temp_view.body_input.text();
            if !body_text.trim().is_empty()
                && temp_view.request_content_type
                    == crate::ui::views::http_request_view::ContentType::Text
            {
                let trimmed = body_text.trim();
                if (trimmed.starts_with('{') && trimmed.ends_with('}'))
                    || (trimmed.starts_with('[') && trimmed.ends_with(']'))
                {
                    if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
                        temp_view.request_content_type =
                            crate::ui::views::http_request_view::ContentType::Json;
                    }
                }
            }

            let mut request = match temp_view.build_request() {
                Ok(r) => r,
                Err(e) => {
                    app.toast_manager
                        .error(format!("Failed to build request: {}", e));
                    return Task::none();
                }
            };

            let mut script_context = ScriptContext::new();
            if let Some(env) = &app.active_environment {
                for (key, value) in &env.variables {
                    script_context
                        .variables
                        .insert(key.clone(), value.clone());
                }
            }

            let pre_request_script = temp_view.scripts.pre_request.clone();
            if !pre_request_script.actions.is_empty() {
                if let Err(e) =
                    ScriptEngine::execute_pre_request(&pre_request_script, &mut request, &mut script_context)
                {
                    app.toast_manager
                        .error(format!("Pre-request script error: {}", e));
                    return Task::none();
                }
                for log in &script_context.logs {
                    app.toast_manager.info(format!("[Pre-request] {}", log));
                }
            }

            let request_url = request.url.clone();
            let request_method = request.method.to_string();

            view.pending_request_data = serde_json::to_string(&request).ok();
            view.update(http_request_view::Message::SetLoading);

            let http_client = if request.config.proxy_url.is_some() || !request.config.tls.verify_ssl {
                let cache_key = format!(
                    "{}|{}",
                    request.config.proxy_url.as_deref().unwrap_or(""),
                    request.config.tls.verify_ssl
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

            let post_response_script = view.scripts.post_response.clone();
            let (task, handle) = Task::perform(
                async move {
                    let response = client::send_request(&http_client, request).await;

                    match response {
                        Ok(mut resp) => {
                            let mut post_ctx = ScriptContext::new();
                            if let Err(e) = ScriptEngine::execute_post_response(
                                &post_response_script,
                                &resp,
                                &mut post_ctx,
                            ) {
                                return Err(crate::error::AppError::Validation(
                                    format!("Post-response script error: {}", e),
                                ));
                            }
                            for log in &post_ctx.logs {
                                log::info!("[Post-response] {}", log);
                            }
                            resp.url = request_url;
                            resp.method = request_method.parse().unwrap_or(crate::http_client::request::HttpMethod::Get);
                            Ok(resp)
                        }
                        Err(e) => Err(e),
                    }
                },
                move |result| {
                    Message::HttpRequestViewMsg(
                        index,
                        http_request_view::Message::ResponseReceived(result),
                    )
                },
            )
            .abortable();
            view.abort_handle = Some(handle);
            task
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
                    let _ = crate::services::history_service::trim(
                        &app.db_conn,
                        crate::persistence::database::DEFAULT_HISTORY_LIMIT,
                    );
                    app.history_view.entries =
                        crate::services::history_service::get_all(&app.db_conn, 200).unwrap_or_default();

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
        http_request_view::Message::CancelRequest => {
            let Some(view) = app.request_tabs.get_mut(index) else {
                return Task::none();
            };
            if let Some(handle) = view.abort_handle.take() {
                handle.abort();
                view.update(http_request_view::Message::SetIdle);
                app.toast_manager.warning("Request cancelled".to_string());
            }
            Task::none()
        }
        http_request_view::Message::SaveScripts => {
            let Some(view) = app.request_tabs.get(index) else {
                return Task::none();
            };
            match view.parse_scripts_from_editors() {
                Ok(_scripts) => {
                    app.toast_manager.success("Scripts saved".to_string());
                    Task::perform(
                        async move { Ok::<(), String>(()) },
                        move |_| Message::HttpRequestViewMsg(index, http_request_view::Message::ScriptsSaved(Ok(()))),
                    )
                }
                Err(e) => {
                    app.toast_manager
                        .error(format!("Invalid scripts: {}", e));
                    Task::none()
                }
            }
        }
        http_request_view::Message::ScriptsSaved(_) => Task::none(),
        http_request_view::Message::ClearKeychainSecrets => {
            if let Some(view) = app.request_tabs.get_mut(index) {
                view.update(http_request_view::Message::ClearKeychainSecrets);
            }
            Task::perform(
                async { Ok::<(), crate::error::AppError>(()) },
                |_| Message::ClearKeychainSecrets,
            )
        }
        other => {
            if let Some(view) = app.request_tabs.get_mut(index) {
                view.update(other);
            }
            Task::none()
        }
    }
}
