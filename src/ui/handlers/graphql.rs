use crate::ui::app::{AstraNovaApp, Message};
use crate::ui::views::graphql_view;
use iced::Task;

pub fn handle_message(app: &mut AstraNovaApp, msg: graphql_view::Message) -> Task<Message> {
    match msg {
        graphql_view::Message::SendRequest => {
            let mut temp_view = app.graphql_view.clone();
            if let Some(env) = &app.active_environment {
                temp_view.apply_environment(env);
            }

            match temp_view.build_request() {
                Ok(_graphql_request) => {
                    let http_request = temp_view.build_http_request();
                    app.graphql_view.update(graphql_view::Message::SetLoading);

                    let http_client = if http_request.config.proxy_url.is_some()
                        || !http_request.config.verify_ssl
                    {
                        match crate::http_client::client::build_client(&http_request.config) {
                            Ok(c) => std::sync::Arc::new(c),
                            Err(e) => {
                                log::error!("Failed to build custom client: {}", e);
                                std::sync::Arc::clone(&app.http_client)
                            }
                        }
                    } else {
                        std::sync::Arc::clone(&app.http_client)
                    };

                    Task::perform(
                        async move {
                            let response = crate::http_client::client::send_request(
                                &http_client,
                                http_request,
                            )
                            .await;

                            match response {
                                Ok(http_response) => {
                                    let graphql_response: crate::protocols::graphql::GraphQLResponse =
                                        serde_json::from_str(&http_response.body)
                                            .unwrap_or_else(|_| crate::protocols::graphql::GraphQLResponse {
                                                data: None,
                                                errors: vec![crate::protocols::graphql::GraphQLError {
                                                    message: format!(
                                                        "Failed to parse GraphQL response: {}",
                                                        &http_response.body[..http_response.body.len().min(200)]
                                                    ),
                                                    locations: vec![],
                                                    path: vec![],
                                                    extensions: None,
                                                }],
                                            });

                                    Ok((
                                        graphql_response,
                                        http_response.status,
                                        http_response.headers,
                                        http_response.duration,
                                        http_response.size,
                                    ))
                                }
                                Err(e) => Err(e),
                            }
                        },
                        move |result| {
                            Message::GraphQLMsg(graphql_view::Message::ResponseReceived(result))
                        },
                    )
                }
                Err(e) => {
                    app.graphql_view
                        .update(graphql_view::Message::ResponseReceived(Err(e)));
                    Task::none()
                }
            }
        }
        graphql_view::Message::IntrospectSchema => {
            let mut temp_view = app.graphql_view.clone();
            if let Some(env) = &app.active_environment {
                temp_view.apply_environment(env);
            }
            let http_request = temp_view.build_introspection_request();

            let http_client = if http_request.config.proxy_url.is_some()
                || !http_request.config.verify_ssl
            {
                match crate::http_client::client::build_client(&http_request.config) {
                    Ok(c) => std::sync::Arc::new(c),
                    Err(e) => {
                        log::error!("Failed to build custom client: {}", e);
                        std::sync::Arc::clone(&app.http_client)
                    }
                }
            } else {
                std::sync::Arc::clone(&app.http_client)
            };

            Task::perform(
                async move {
                    let response =
                        crate::http_client::client::send_request(&http_client, http_request)
                            .await?;

                    let introspection: crate::protocols::graphql_schema::IntrospectionResponse =
                        serde_json::from_str(&response.body).map_err(|e| {
                            crate::error::AppError::Serialization(format!(
                                "Failed to parse introspection response: {}",
                                e
                            ))
                        })?;

                    crate::protocols::graphql_schema::parse_introspection_response(&introspection)
                },
                move |result| {
                    Message::GraphQLMsg(graphql_view::Message::SchemaReceived(result))
                },
            )
        }
        graphql_view::Message::SaveToHistory => {
            let view = &app.graphql_view;
            let url = view.url_input.clone();
            let query = view.query_input.text();
            let variables = view.variables_input.text();
            let operation_name = view.operation_name.clone();

            let graphql_request = crate::protocols::graphql::GraphQLRequest {
                query: query.clone(),
                variables: if variables.trim().is_empty() {
                    None
                } else {
                    crate::protocols::graphql::parse_variables(&variables).ok()
                },
                operation_name: if operation_name.trim().is_empty() {
                    None
                } else {
                    Some(operation_name.clone())
                },
            };

            let request_data = serde_json::to_string(&graphql_request).ok();

            let response_data = view.last_response.as_ref().and_then(|r| {
                serde_json::to_string(r).ok()
            });

            let conn = &app.db_conn;
            let method = "GRAPHQL";
            let status = view.status_code.map(|s| s as i64);
            let duration_ms = view
                .response_duration
                .map(|d| d.as_millis() as i64);

            let result = conn.execute(
                "INSERT INTO request_history (method, url, status, duration_ms, timestamp, request_data, response_data) VALUES (?1, ?2, ?3, ?4, datetime('now'), ?5, ?6)",
                rusqlite::params![
                    method,
                    url,
                    status,
                    duration_ms,
                    request_data,
                    response_data,
                ],
            );

            match result {
                Ok(_) => {
                    app.graphql_view
                        .update(graphql_view::Message::SavedToHistory(Ok(())));
                    let entries = crate::services::history_service::get_all(&app.db_conn, 200);
                    app.history_view.entries = entries;
                }
                Err(e) => {
                    app.graphql_view.update(graphql_view::Message::SavedToHistory(
                        Err(crate::error::AppError::Database(e.to_string())),
                    ));
                }
            }
            Task::none()
        }
        graphql_view::Message::SaveToCollection(collection_id, folder_id) => {
            let view = &app.graphql_view;
            let url = view.url_input.clone();
            let query = view.query_input.text();
            let variables = view.variables_input.text();
            let operation_name = view.operation_name.clone();
            let headers: Vec<(String, String)> = view
                .headers_editor
                .entries
                .iter()
                .filter(|h| !h.key.is_empty())
                .map(|h| (h.key.clone(), h.value.clone()))
                .collect();

            let graphql_body = serde_json::to_string(
                &crate::protocols::graphql::GraphQLRequest {
                    query,
                    variables: if variables.trim().is_empty() {
                        None
                    } else {
                        crate::protocols::graphql::parse_variables(&variables).ok()
                    },
                    operation_name: if operation_name.trim().is_empty() {
                        None
                    } else {
                        Some(operation_name)
                    },
                },
            )
            .ok();

            let auth_json = serde_json::to_string(&view.auth).ok();

            let result = crate::services::collection_service::save_request(
                &app.db_conn,
                collection_id,
                folder_id,
                &format!("GraphQL Request - {}", url.chars().take(50).collect::<String>()),
                "POST",
                &url,
                &headers,
                graphql_body.as_deref(),
                &crate::persistence::database::CollectionBodyType::Graphql,
                &crate::persistence::database::CollectionAuthType::None,
                auth_json.as_deref(),
                &[],
                None,
            );

            match result {
                Ok(_) => {
                    app.graphql_view.update(
                        graphql_view::Message::SavedToCollection(Ok(())),
                    );
                    let cols = crate::services::collection_service::get_all(&app.db_conn);
                    app.collection_view.sync_collections(&cols);
                }
                Err(e) => {
                    app.graphql_view.update(
                        graphql_view::Message::SavedToCollection(Err(e)),
                    );
                }
            }
            Task::none()
        }
        other => {
            app.graphql_view.update(other);
            Task::none()
        }
    }
}
