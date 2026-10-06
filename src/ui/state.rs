use crate::protocols::websocket::WsSender;
use crate::ui::views::mock_server_view::MockServerView;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

type HttpStreamReceiver =
    Arc<Mutex<Option<mpsc::UnboundedReceiver<crate::http_client::response::HttpStreamEvent>>>>;

pub struct WsState {
    pub sender: Option<WsSender>,
    pub receiver: Option<Arc<Mutex<Option<mpsc::UnboundedReceiver<crate::protocols::websocket::WsEvent>>>>>,
    pub shutdown: Option<mpsc::UnboundedSender<()>>,
    pub write_handle: Option<Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>>,
    pub read_handle: Option<Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>>,
    pub ping_handle: Option<Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>>,
    pub connection_id: u64,
}

impl WsState {
    pub fn new() -> Self {
        Self {
            sender: None,
            receiver: None,
            shutdown: None,
            write_handle: None,
            read_handle: None,
            ping_handle: None,
            connection_id: 0,
        }
    }

    pub fn abort_handles(&mut self) {
        for handle in [&mut self.write_handle, &mut self.read_handle, &mut self.ping_handle] {
            if let Some(h) = handle.take() {
                if let Some(handle) = h.lock().ok().and_then(|mut h| h.take()) {
                    handle.abort();
                }
            }
        }
    }

    pub fn shutdown(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        self.abort_handles();
    }

    pub fn reset(&mut self) {
        self.sender = None;
        self.receiver = None;
    }
}

impl Default for WsState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct HttpStreamState {
    pub receivers: HashMap<usize, (u64, HttpStreamReceiver)>,
    pub next_id: u64,
}

impl HttpStreamState {
    pub fn new() -> Self {
        Self {
            receivers: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn next_stream_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

impl Default for HttpStreamState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MockState {
    pub handles: HashMap<i32, crate::protocols::mock_server::MockServerHandle>,
    pub view: MockServerView,
}

impl MockState {
    pub fn new(view: MockServerView) -> Self {
        Self {
            handles: HashMap::new(),
            view,
        }
    }
}

pub struct AiStreamState {
    pub receiver: Option<Arc<tokio::sync::Mutex<Option<tokio::sync::mpsc::Receiver<String>>>>>,
    pub stream_id: u64,
}

impl AiStreamState {
    pub fn new() -> Self {
        Self {
            receiver: None,
            stream_id: 0,
        }
    }

    pub fn set_receiver(
        &mut self,
        rx: Arc<tokio::sync::Mutex<Option<tokio::sync::mpsc::Receiver<String>>>>,
    ) {
        self.stream_id += 1;
        self.receiver = Some(rx);
    }
}

impl Default for AiStreamState {
    fn default() -> Self {
        Self::new()
    }
}
