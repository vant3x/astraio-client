use crate::ui::app::AstraioApp;
use crate::ui::message::Message;
use crate::ui::views::mock_server_view;
use iced::Task;
pub fn handle_message(app: &mut AstraioApp, msg: mock_server_view::Message) -> Task<Message> {
    match msg {
        mock_server_view::Message::ToggleAddServer => {
            app.mock.view.show_add_server = !app.mock.view.show_add_server;
            if !app.mock.view.show_add_server {
                app.mock.view.new_server_name.clear();
            }
        }
        mock_server_view::Message::NewServerNameChanged(name) => {
            app.mock.view.new_server_name = name;
        }
        mock_server_view::Message::CreateServer(name) => {
            if name.trim().is_empty() {
                return Task::none();
            }
            let port = find_free_port();
            match crate::services::mock_server_service::create_and_refresh(
                &app.db_conn,
                name.trim(),
                port,
            ) {
                Ok(servers) => {
                    app.mock.view.sync_servers(&servers);
                    app.mock.view.new_server_name = String::new();
                    app.mock.view.show_add_server = false;
                    app.toast_manager.success(format!(
                        "Created '{}' on port {}",
                        name.trim(),
                        port
                    ));
                }
                Err(e) => {
                    log::error!("Error creating mock server: {e}");
                    app.toast_manager.error(format!("Failed to create: {e}"));
                }
            }
        }
        mock_server_view::Message::SelectServer(id) => {
            app.mock.view.selected_server_id = id;
            app.mock.view.endpoint_edit = None;
        }
        mock_server_view::Message::DeleteServer(id) => {
            if let Some(handle) = app.mock.handles.remove(&id) {
                crate::protocols::mock_server::stop_mock_server(handle);
                app.mock.view.statuses.remove(&id);
            }
            match crate::services::mock_server_service::delete_and_refresh(&app.db_conn, id) {
                Ok(servers) => {
                    app.mock.view.sync_servers(&servers);
                    if app.mock.view.selected_server_id == Some(id) {
                        app.mock.view.selected_server_id = None;
                    }
                    app.toast_manager.success("Mock server deleted");
                }
                Err(e) => log::error!("Error deleting mock server: {e}"),
            }
        }
        mock_server_view::Message::StartServer(id) => {
            let config = match app.mock.view.servers.iter().find(|s| s.id == id) {
                Some(c) => c.clone(),
                None => return Task::none(),
            };

            log::info!(
                "[Mock] Starting server '{}' on port {}",
                config.name,
                config.port
            );
            app.mock.view.statuses.insert(
                id,
                crate::protocols::mock_server::MockServerStatus::Starting,
            );

            let server_id = id;
            return Task::perform(
                async move { crate::protocols::mock_server::start_mock_server(&config).await },
                move |result| match result {
                    Ok((handle, actual_port)) => {
                        log::info!("[Mock] Server started successfully on port {actual_port}");
                        Message::MockServerStarted(server_id, handle, actual_port)
                    }
                    Err(e) => {
                        log::error!("[Mock] Failed to start server: {e}");
                        Message::MockServerStartError(server_id, e)
                    }
                },
            );
        }
        mock_server_view::Message::StopServer(id) => {
            if let Some(handle) = app.mock.handles.remove(&id) {
                crate::protocols::mock_server::stop_mock_server(handle);
            }
            app.mock.view
                .statuses
                .insert(id, crate::protocols::mock_server::MockServerStatus::Stopped);
            app.toast_manager.info("Mock server stopped");
        }
        mock_server_view::Message::AddEndpoint(server_id) => {
            let body = r#"{"message": "Hello, World!"}"#.to_string();
            app.mock.view.endpoint_edit = Some(mock_server_view::EndpointEditState {
                mock_server_id: server_id,
                endpoint_id: None,
                method: "GET".to_string(),
                path: "/".to_string(),
                status: "200".to_string(),
                body: iced::widget::text_editor::Content::with_text(&body),
                delay_ms: "0".to_string(),
                headers: crate::ui::components::key_value_editor::KeyValueEditor::new(
                    "Add Header".to_string(),
                ),
            });
        }
        mock_server_view::Message::EditEndpoint(endpoint_id) => {
            let server_id = app.mock.view.selected_server_id.unwrap_or(0);
            if let Some(server) = app
                .mock.view
                .servers
                .iter()
                .find(|s| s.id == server_id)
            {
                if let Some(ep) = server.endpoints.iter().find(|e| e.id == endpoint_id) {
                    let body_text = ep.body.clone().unwrap_or_default();
                    let headers_entries = ep
                        .headers
                        .iter()
                        .enumerate()
                        .map(
                            |(i, (k, v))| crate::ui::components::key_value_editor::KeyValueEntry {
                                id: i,
                                key: k.clone(),
                                value: v.clone(),
                                secret: false,
                            },
                        )
                        .collect();
                    app.mock.view.endpoint_edit =
                        Some(mock_server_view::EndpointEditState {
                            mock_server_id: server_id,
                            endpoint_id: Some(ep.id),
                            method: ep.method.clone(),
                            path: ep.path.clone(),
                            status: ep.status.to_string(),
                            body: iced::widget::text_editor::Content::with_text(&body_text),
                            delay_ms: ep.delay_ms.to_string(),
                            headers: {
                                let mut editor =
                                    crate::ui::components::key_value_editor::KeyValueEditor::new(
                                        "Add Header".to_string(),
                                    );
                                editor.entries = headers_entries;
                                editor
                            },
                        });
                }
            }
        }
        mock_server_view::Message::EndpointMethodSelected(method) => {
            if let Some(ref mut edit) = app.mock.view.endpoint_edit {
                edit.method = method;
            }
        }
        mock_server_view::Message::EndpointPathChanged(path) => {
            if let Some(ref mut edit) = app.mock.view.endpoint_edit {
                edit.path = path;
            }
        }
        mock_server_view::Message::EndpointStatusChanged(status) => {
            if let Some(ref mut edit) = app.mock.view.endpoint_edit {
                edit.status = status;
            }
        }
        mock_server_view::Message::EndpointBodyAction(action) => {
            if let Some(ref mut edit) = app.mock.view.endpoint_edit {
                edit.body.perform(action);
            }
        }
        mock_server_view::Message::EndpointDelayChanged(delay) => {
            if let Some(ref mut edit) = app.mock.view.endpoint_edit {
                edit.delay_ms = delay;
            }
        }
        mock_server_view::Message::EndpointHeadersEditor(msg) => {
            if let Some(ref mut edit) = app.mock.view.endpoint_edit {
                edit.headers.update(msg);
            }
        }
        mock_server_view::Message::SaveEndpoint => {
            if let Some(edit) = app.mock.view.endpoint_edit.take() {
                let status_code: u16 = edit.status.parse().unwrap_or(200);
                let body_text = edit.body.text();
                let body_opt = if body_text.is_empty() {
                    None
                } else {
                    Some(body_text.as_str())
                };
                let delay_ms: u64 = edit.delay_ms.parse().unwrap_or(0);
                let headers: Vec<(String, String)> = edit
                    .headers
                    .entries
                    .iter()
                    .filter(|h| !h.key.is_empty())
                    .map(|h| (h.key.clone(), h.value.clone()))
                    .collect();

                let result = if let Some(ep_id) = edit.endpoint_id {
                    crate::services::mock_server_service::update_endpoint(
                        &app.db_conn,
                        ep_id,
                        edit.mock_server_id,
                        &edit.method,
                        &edit.path,
                        status_code,
                        &headers,
                        body_opt,
                        delay_ms,
                    )
                } else {
                    crate::services::mock_server_service::add_endpoint(
                        &app.db_conn,
                        edit.mock_server_id,
                        &edit.method,
                        &edit.path,
                        status_code,
                        &headers,
                        body_opt,
                        delay_ms,
                    )
                };

                match result {
                    Ok(_) => {
                        let servers = crate::services::mock_server_service::get_all(&app.db_conn)
                            .unwrap_or_default();
                        app.mock.view.sync_servers(&servers);
                        app.toast_manager.success("Endpoint saved");
                    }
                    Err(e) => {
                        log::error!("Error saving endpoint: {e}");
                        app.toast_manager.error(format!("Failed to save: {e}"));
                    }
                }
            }
        }
        mock_server_view::Message::CancelEndpointEdit => {
            app.mock.view.endpoint_edit = None;
        }
        mock_server_view::Message::DeleteEndpoint(endpoint_id, server_id) => {
            match crate::services::mock_server_service::delete_endpoint(
                &app.db_conn,
                endpoint_id,
                server_id,
            ) {
                Ok(_) => {
                    let servers = crate::services::mock_server_service::get_all(&app.db_conn)
                        .unwrap_or_default();
                    app.mock.view.sync_servers(&servers);
                    app.toast_manager.success("Endpoint deleted");
                }
                Err(e) => log::error!("Error deleting endpoint: {e}"),
            }
        }
        mock_server_view::Message::EndpointSearchChanged(query) => {
            app.mock.view.endpoint_search = query;
        }
        mock_server_view::Message::ClearLogs => {
            app.mock.view.logs.clear();
        }
        mock_server_view::Message::AiMockDescriptionChanged(desc) => {
            app.mock.view.ai_mock_description = desc;
        }
        mock_server_view::Message::AiMockGenerate(description) => {
            return handle_ai_mock_generate(app, &description);
        }
        mock_server_view::Message::AiMockResult(body) => {
            app.mock.view.ai_mock_generating = false;
            if let Some(ref mut edit) = app.mock.view.endpoint_edit {
                edit.body = iced::widget::text_editor::Content::with_text(&body);
                app.toast_manager.success("Mock data generated");
            }
        }
        mock_server_view::Message::AiMockError(err) => {
            app.mock.view.ai_mock_generating = false;
            app.toast_manager.error(format!("AI error: {err}"));
        }
    }

    Task::none()
}

fn find_free_port() -> u16 {
    use std::net::TcpListener;
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to find free port");
    listener
        .local_addr()
        .expect("listener must have local address")
        .port()
}

fn handle_ai_mock_generate(app: &mut AstraioApp, description: &str) -> Task<Message> {
    let config = match app
        .ai_view
        .active_provider_index
        .and_then(|idx| app.ai_view.providers.get(idx))
    {
        Some(c) => c.clone(),
        None => {
            app.toast_manager
                .error("No AI provider configured. Open AI Settings first.");
            return Task::none();
        }
    };

    if app
        .secret_store
        .get_secret("ai", &format!("{}_{}", config.provider, config.name), "api_key")
        .ok()
        .flatten()
        .is_none()
        && config.provider != crate::ai::types::AiProvider::Ollama
    {
        app.toast_manager
            .error("No API key configured for this AI provider.");
        return Task::none();
    }

    app.mock.view.ai_mock_generating = true;

    let prompt = format!(
        "Generate a realistic JSON mock response for an API endpoint. \
         Description: {description}\n\n\
         Return ONLY valid JSON, no explanation, no markdown code fences. \
         The JSON should be a realistic example response matching the description."
    );

    let ai_service = app.ai_service.clone();

    Task::perform(
        async move {
            let service = ai_service.lock().await;
            let request = crate::ai::types::AiChatRequest {
                model: config.model.clone(),
                messages: vec![crate::ai::types::AiChatMessage {
                    role: crate::ai::types::AiRole::User,
                    content: prompt,
                }],
                max_tokens: Some(2048),
                temperature: Some(0.7),
                stream: false,
            };
            let result = service.chat(&config, request).await;
            drop(service);
            result
        },
        |result| match result {
            Ok(response) => {
                let body = response.content.trim().to_string();
                Message::MockServerMsg(crate::ui::views::mock_server_view::Message::AiMockResult(
                    body,
                ))
            }
            Err(e) => Message::MockServerMsg(
                crate::ui::views::mock_server_view::Message::AiMockError(e.to_string()),
            ),
        },
    )
}
