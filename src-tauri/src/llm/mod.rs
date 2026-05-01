// 데스크톱(Linux/Windows)은 Claude Code CLI(headless, Haiku)를 subprocess로 호출한다.
// 모바일/그 외 타깃의 fallback이 mock으로 가므로, mock 모듈은 그쪽으로만 컴파일한다.
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub mod mock;

#[cfg(any(target_os = "linux", target_os = "windows"))]
pub mod claude_code_backend;

use async_trait::async_trait;
use shared::{IpcError, SummaryRequest, SummaryResponse};

#[async_trait]
pub trait LlmSummarizer: Send + Sync {
    async fn summarize(&self, req: SummaryRequest) -> Result<SummaryResponse, IpcError>;
}
