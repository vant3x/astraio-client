use crate::protocols::websocket::{connect_ws, WsRequest, WsStatus};
use crate::ui::app::{AstraNovaApp, Message};
use crate::ui::views::websocket_view;
use iced::Task;
use std::sync::{Arc, Mutex};

pub fn handle_message(app: &mut AstraNovaApp, message: websocket_view::Message) -> Task<Message> {
    match message {
        websocket_view::Message::UrlChanged(url) => {
            app.websocket_view.url = url;
            Task::none()
        }
        websocket_view::Message::HeaderKeyChanged(key) => {
            app.websocket_view.header_key = key;
            Task::none()
        }
        websocket_view::Message::HeaderValueChanged(value) => {
            app.websocket_view.header_value = value;
            Task::none()
        }
        websocket_view::Message::AddHeader => {
            let key = app.websocket_view.header_key.clone();
            let value = app.websocket_view.header_value.clone();
            if !key.is_empty() {
                app.websocket_view.headers.push((key, value));
                app.websocket_view.header_key.clear();
                app.websocket_view.header_value.clear();
            }
            Task::none()
        }
        websocket_view::Message::RemoveHeader(index) => {
            if index < app.websocket_view.headers.len() {
                app.websocket_view.headers.remove(index);
            }
            Task::none()
        }
        websocket_view::Message::Connect => handle_connect(app),
        websocket_view::Message::Disconnect => handle_disconnect(app),
        websocket_view::Message::Disconnected(reason) => handle_disconnected(app, reason),
        websocket_view::Message::SendMessage(text) => {
            if let Some(sender) = &app.websocket_view.ws_sender {
                if !text.is_empty() {
                    let _ = sender.send(&text);
                    app.websocket_view.messages.push(
                        crate::protocols::websocket::WsMessage::outgoing(text),
                    );
                    app.websocket_view.input.clear();
                }
            }
            Task::none()
        }
        websocket_view::Message::SendBinary(hex) => {
            if let Some(sender) = &app.websocket_view.ws_sender {
                if !hex.is_empty() {
                    let bytes: Vec<u8> = hex
                        .split_whitespace()
                        .filter_map(|s| u8::from_str_radix(s, 16).ok())
                        .collect();
                    if !bytes.is_empty() {
                        let _ = sender.send_binary(bytes.clone());
                        let hex_display = bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                        app.websocket_view.messages.push(
                            crate::protocols::websocket::WsMessage {
                                direction: ">".to_string(),
                                message_type: crate::protocols::websocket::WsMessageType::Binary,
                                data: hex_display,
                                timestamp: crate::utils::timestamp_seconds(),
                            },
                        );
                        app.websocket_view.hex_input.clear();
                    }
                }
            }
            Task::none()
        }
        websocket_view::Message::SendPing => {
            if let Some(sender) = &app.websocket_view.ws_sender {
                let _ = sender.send("ping");
                app.websocket_view.messages.push(
                    crate::protocols::websocket::WsMessage::incoming(
                        crate::protocols::websocket::WsMessageType::Ping,
                        "ping".to_string(),
                    ),
                );
            }
            Task::none()
        }
        websocket_view::Message::SendClose(reason) => {
            if let Some(_sender) = &app.websocket_view.ws_sender {
                let _reason = if reason.is_empty() {
                    "Goodbye".to_string()
                } else {
                    reason
                };
                let _ = handle_disconnect(app);
            }
            Task::none()
        }
        websocket_view::Message::InputChanged(input) => {
            app.websocket_view.input = input;
            Task::none()
        }
        websocket_view::Message::HexInputChanged(hex) => {
            app.websocket_view.hex_input = hex;
            Task::none()
        }
        websocket_view::Message::MessageTypeSelected(filter) => {
            app.websocket_view.message_type_filter = filter;
            Task::none()
        }
        websocket_view::Message::ToggleHeaders => {
            app.websocket_view.show_headers = !app.websocket_view.show_headers;
            Task::none()
        }
        websocket_view::Message::ToggleAutoReconnect => {
            app.websocket_view.auto_reconnect = !app.websocket_view.auto_reconnect;
            if app.websocket_view.auto_reconnect {
                log::info!("WebSocket auto-reconnect enabled");
            } else {
                log::info!("WebSocket auto-reconnect disabled");
            }
            Task::none()
        }
        websocket_view::Message::ReconnectDelayChanged(delay) => {
            if let Ok(d) = delay.parse::<u64>() {
                app.websocket_view.reconnect_delay_ms = d;
            }
            Task::none()
        }
        websocket_view::Message::MaxRetriesChanged(retries) => {
            if let Ok(r) = retries.parse::<u32>() {
                app.websocket_view.max_retries = r;
            }
            Task::none()
        }
        websocket_view::Message::SearchChanged(query) => {
            app.websocket_view.search_query = query;
            Task::none()
        }
        websocket_view::Message::SubprotocolChanged(sub) => {
            app.websocket_view.subprotocol = sub;
            Task::none()
        }
        websocket_view::Message::ClearMessages => {
            app.websocket_view.messages.clear();
            Task::none()
        }
        websocket_view::Message::ConnectedWithSender(_) => Task::none(),
    }
}

fn handle_connect(app: &mut AstraNovaApp) -> Task<Message> {
    let url = app.websocket_view.url.clone();
    let headers = app.websocket_view.headers.clone();
    let subprotocol = if app.websocket_view.subprotocol.is_empty() {
        None
    } else {
        Some(app.websocket_view.subprotocol.clone())
    };

    if url.is_empty() {
        app.websocket_view.status =
            WsStatus::Error("URL is required".to_string());
        return Task::none();
    }

    app.websocket_view.status = WsStatus::Connecting;
    app.websocket_view.current_retries = 0;

    log::info!("Connecting to WebSocket: {}", url);

    Task::perform(
        async move {
            let request = WsRequest {
                url,
                headers,
                subprotocol,
            };
            connect_ws(&request).await
        },
        |result| match result {
            Ok(conn) => Message::WsConnected(
                conn.sender,
                Arc::new(Mutex::new(Some(conn.receiver))),
                conn.shutdown_tx,
                Arc::new(Mutex::new(Some(conn.write_handle))),
                Arc::new(Mutex::new(Some(conn.read_handle))),
            ),
            Err(e) => {
                log::error!("WebSocket connection failed: {}", e);
                Message::WebSocketMsg(websocket_view::Message::Disconnected(e.to_string()))
            }
        },
    )
}

fn handle_disconnect(app: &mut AstraNovaApp) -> Task<Message> {
    log::info!("Disconnecting WebSocket");
    if let Some(shutdown) = app.ws_shutdown.take() {
        let _ = shutdown.send(());
    }
    app.ws_sender = None;
    app.ws_receiver = None;
    app.websocket_view.status = WsStatus::Disconnected;
    app.websocket_view.current_retries = 0;
    Task::none()
}

fn handle_disconnected(app: &mut AstraNovaApp, reason: String) -> Task<Message> {
    if reason == "cleared" {
        app.websocket_view.messages.clear();
        return Task::none();
    }

    app.ws_sender = None;
    app.ws_receiver = None;
    app.websocket_view.status = WsStatus::Disconnected;

    if app.websocket_view.auto_reconnect
        && app.websocket_view.current_retries < app.websocket_view.max_retries
    {
        app.websocket_view.current_retries += 1;
        let delay = app.websocket_view.reconnect_delay_ms;
        let retries = app.websocket_view.current_retries;
        let max = app.websocket_view.max_retries;

        log::info!(
            "Auto-reconnect {}/{} in {}ms",
            retries,
            max,
            delay
        );

        app.websocket_view.status = WsStatus::Connecting;

        let url = app.websocket_view.url.clone();
        let headers = app.websocket_view.headers.clone();
        let subprotocol = if app.websocket_view.subprotocol.is_empty() {
            None
        } else {
            Some(app.websocket_view.subprotocol.clone())
        };

        return Task::perform(
            async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                let request = WsRequest {
                    url,
                    headers,
                    subprotocol,
                };
                connect_ws(&request).await
            },
            |result| match result {
                Ok(conn) => Message::WsConnected(
                    conn.sender,
                    Arc::new(Mutex::new(Some(conn.receiver))),
                    conn.shutdown_tx,
                    Arc::new(Mutex::new(Some(conn.write_handle))),
                    Arc::new(Mutex::new(Some(conn.read_handle))),
                ),
                Err(e) => {
                    log::error!("WebSocket reconnection failed: {}", e);
                    Message::WebSocketMsg(websocket_view::Message::Disconnected(e.to_string()))
                }
            },
        );
    }

    if !reason.is_empty() && reason != "Connection closed" {
        log::warn!("WebSocket disconnected: {}", reason);
    }

    Task::none()
}

pub fn handle_ws_event(
    app: &mut AstraNovaApp,
    event: crate::protocols::websocket::WsEvent,
) -> Task<Message> {
    match event {
        crate::protocols::websocket::WsEvent::Message(msg) => {
            app.websocket_view.messages.push(msg);
            Task::none()
        }
        crate::protocols::websocket::WsEvent::Connected => {
            log::info!("WebSocket connected");
            app.websocket_view.status = WsStatus::Connected;
            Task::none()
        }
        crate::protocols::websocket::WsEvent::Disconnected(reason) => {
            handle_disconnected(app, reason)
        }
        crate::protocols::websocket::WsEvent::Error(e) => {
            log::error!("WebSocket error: {}", e);
            app.websocket_view.status = WsStatus::Error(e);
            Task::none()
        }
    }
}

pub fn handle_ws_connected(
    app: &mut AstraNovaApp,
    sender: crate::protocols::websocket::WsSender,
    receiver: Arc<Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<crate::protocols::websocket::WsEvent>>>>,
    shutdown_tx: Option<tokio::sync::mpsc::UnboundedSender<()>>,
    write_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    read_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
) {
    app.ws_sender = Some(sender);
    app.ws_receiver = Some(receiver);
    app.ws_shutdown = shutdown_tx;
    app.ws_write_handle = Some(write_handle);
    app.ws_read_handle = Some(read_handle);
    app.websocket_view.status = WsStatus::Connected;
    app.websocket_view.current_retries = 0;
    log::info!("WebSocket connection established");
}
