use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use shared::{EventEnvelope, Platform};
use tauri::async_runtime::JoinHandle;
use tokio::sync::{broadcast, OnceCell};

use crate::llm::LlmSummarizer;

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
    /// LLM 모델은 앱 setup에서 비동기 1회 로드된다.
    /// 데스크톱(Linux/Windows)에서 Qwen3.6 GGUF가 채워지고,
    /// 모바일/macOS는 비어 있을 수 있다.
    pub summarizer: OnceCell<Arc<dyn LlmSummarizer>>,
    /// 진행 중인 OAuth 흐름의 abort 핸들. 사용자가 "취소"를 누르거나
    /// 새 흐름을 시작하면 기존 핸들을 abort한다.
    pub oauth_handle: Mutex<Option<ConnectionHandle>>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            connections: Mutex::new(HashMap::new()),
            mock_handle: Mutex::new(None),
            event_tx: tx,
            summarizer: OnceCell::new(),
            oauth_handle: Mutex::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
