use std::collections::HashMap;
use std::sync::Mutex;

use shared::{EventEnvelope, Platform};
use tauri::async_runtime::JoinHandle;
use tokio::sync::broadcast;

pub struct ConnectionHandle {
    handle: JoinHandle<()>,
}

impl ConnectionHandle {
    pub fn new(handle: JoinHandle<()>) -> Self {
        Self { handle }
    }

    pub fn abort(self) {
        self.handle.abort();
    }
}

pub struct AppState {
    pub connections: Mutex<HashMap<Platform, ConnectionHandle>>,
    pub mock_handle: Mutex<Option<ConnectionHandle>>,
    pub event_tx: broadcast::Sender<EventEnvelope>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            connections: Mutex::new(HashMap::new()),
            mock_handle: Mutex::new(None),
            event_tx: tx,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
