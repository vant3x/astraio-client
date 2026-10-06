use crate::ui::message::Message;
use iced::futures::stream::BoxStream;
use iced::futures::{self, StreamExt as _};
use iced_futures::subscription::{EventStream, Recipe};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub(crate) struct AiStreamRecipe {
    pub receiver: Arc<tokio::sync::Mutex<Option<mpsc::Receiver<String>>>>,
    pub stream_id: u64,
}

impl Recipe for AiStreamRecipe {
    type Output = Message;

    fn hash(&self, state: &mut iced_futures::subscription::Hasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<AiStreamRecipe>().hash(state);
        self.stream_id.hash(state);
    }

    fn stream(self: Box<Self>, _input: EventStream) -> BoxStream<'static, Message> {
        use futures::stream::StreamExt;

        let receiver_arc = self.receiver;

        async fn poll_next(
            arc: &Arc<tokio::sync::Mutex<Option<mpsc::Receiver<String>>>>,
        ) -> Option<Message> {
            let mut guard = arc.lock().await;
            let mut receiver = guard.take()?;
            drop(guard);

            let result = tokio::time::timeout(
                std::time::Duration::from_secs(300),
                receiver.recv(),
            )
            .await;

            {
                let mut guard = arc.lock().await;
                match result {
                    Ok(Some(text)) => {
                        *guard = Some(receiver);
                        Some(Message::AiMsg(
                            crate::ui::views::ai_chat_view::Message::ReceiveStreamChunk(text),
                        ))
                    }
                    Ok(None) => {
                        // Stream finished normally
                        Some(Message::AiMsg(
                            crate::ui::views::ai_chat_view::Message::ReceiveStreamEnd,
                        ))
                    }
                    Err(_) => {
                        // Timeout - receiver took too long, force end
                        log::warn!("AI stream timed out after 5 minutes");
                        Some(Message::AiMsg(
                            crate::ui::views::ai_chat_view::Message::ReceiveError(
                                "Stream timed out".to_string(),
                            ),
                        ))
                    }
                }
            }
        }

        futures::stream::unfold(receiver_arc, |arc| async move {
            poll_next(&arc).await.map(|msg| (msg, arc))
        })
        .boxed()
    }
}

pub(crate) struct WsRecipe {
    pub receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<crate::protocols::websocket::WsEvent>>>>,
    pub connection_id: u64,
}

impl Recipe for WsRecipe {
    type Output = Message;

    fn hash(&self, state: &mut iced_futures::subscription::Hasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<WsRecipe>().hash(state);
        self.connection_id.hash(state);
    }

    fn stream(self: Box<Self>, _input: EventStream) -> BoxStream<'static, Message> {
        let receiver_arc = self.receiver;
        futures::stream::unfold(receiver_arc, |arc| async move {
            let mut receiver = {
                let mut guard = arc.lock().ok()?;
                guard.take()?
            };
            let event = receiver.recv().await?;
            if let Ok(mut guard) = arc.lock() {
                *guard = Some(receiver);
            }
            Some((Message::WsEvent(event), arc))
        })
        .boxed()
    }
}

pub(crate) struct HttpStreamRecipe {
    pub receiver:
        Arc<Mutex<Option<mpsc::UnboundedReceiver<crate::http_client::response::HttpStreamEvent>>>>,
    pub tab_index: usize,
    pub stream_id: u64,
}

impl Recipe for HttpStreamRecipe {
    type Output = Message;

    fn hash(&self, state: &mut iced_futures::subscription::Hasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<HttpStreamRecipe>().hash(state);
        self.stream_id.hash(state);
    }

    fn stream(self: Box<Self>, _input: EventStream) -> BoxStream<'static, Message> {
        let receiver_arc = self.receiver;
        let tab_index = self.tab_index;
        futures::stream::unfold(receiver_arc, move |arc| async move {
            let mut receiver = {
                let mut guard = arc.lock().ok()?;
                guard.take()?
            };
            let event = receiver.recv().await?;
            if let Ok(mut guard) = arc.lock() {
                *guard = Some(receiver);
            }
            Some((Message::HttpStreamChunk(tab_index, event), arc))
        })
        .boxed()
    }
}

pub(crate) struct MenuEventRecipe;

impl Recipe for MenuEventRecipe {
    type Output = Message;

    fn hash(&self, state: &mut iced_futures::subscription::Hasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<MenuEventRecipe>().hash(state);
    }

    fn stream(self: Box<Self>, _input: EventStream) -> BoxStream<'static, Message> {
        use std::time::Duration;

        let menu_interval = Duration::from_millis(50);
        let log_interval = Duration::from_secs(1);

        futures::stream::unfold(
            (
                tokio::time::Instant::now() + menu_interval,
                tokio::time::Instant::now() + log_interval,
            ),
            move |(next_menu_tick, next_log_tick)| async move {
                let now = tokio::time::Instant::now();

                let sleep_dur = std::cmp::min(
                    next_menu_tick.saturating_duration_since(now),
                    next_log_tick.saturating_duration_since(now),
                );
                if !sleep_dur.is_zero() {
                    tokio::time::sleep(sleep_dur).await;
                }
                let now = tokio::time::Instant::now();

                if now >= next_menu_tick {
                    if let Some(msg) = muda::MenuEvent::receiver()
                        .try_recv()
                        .ok()
                        .and_then(|event| crate::ui::menu::handle_menu_event(&event))
                    {
                        return Some((msg, (now + menu_interval, next_log_tick)));
                    }
                }

                if now >= next_log_tick {
                    return Some((
                        Message::PollMockServerLogs,
                        (next_menu_tick, now + log_interval),
                    ));
                }

                Some((Message::NoOp, (now + menu_interval, next_log_tick)))
            },
        )
        .boxed()
    }
}

pub(crate) struct DevicePollRecipe {
    pub tab_index: usize,
    pub device_code: String,
    pub client_id: String,
    pub client_secret: String,
    pub token_url: String,
    pub interval_secs: u64,
    pub http_client: Arc<reqwest::Client>,
}

impl Recipe for DevicePollRecipe {
    type Output = Message;

    fn hash(&self, state: &mut iced_futures::subscription::Hasher) {
        use std::hash::Hash;
        std::any::TypeId::of::<DevicePollRecipe>().hash(state);
        self.tab_index.hash(state);
        self.device_code.hash(state);
    }

    fn stream(self: Box<Self>, _input: EventStream) -> BoxStream<'static, Message> {
        let tab_index = self.tab_index;
        let device_code = self.device_code;
        let client_id = self.client_id;
        let client_secret = self.client_secret;
        let token_url = self.token_url;
        let interval = std::time::Duration::from_secs(self.interval_secs.max(5));
        let http_client = self.http_client;

        futures::stream::unfold((), move |()| {
            let device_code = device_code.clone();
            let client_id = client_id.clone();
            let client_secret = client_secret.clone();
            let token_url = token_url.clone();
            let http_client = http_client.clone();
            async move {
                tokio::time::sleep(interval).await;
                let result = crate::data::oauth2::poll_device_token(
                    &http_client,
                    &token_url,
                    &device_code,
                    &client_id,
                    &client_secret,
                )
                .await;
                Some((Message::OAuth2DeviceTokenPoll(tab_index, result), ()))
            }
        })
        .boxed()
    }
}
