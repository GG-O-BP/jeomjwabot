// 데스크톱(Linux/Windows)에서는 Qwen3.6 GGUF만 사용한다.
// 모바일/그 외 타깃의 fallback이 mock으로 가므로, mock 모듈은 그쪽으로만 컴파일한다.
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub mod mock;

#[cfg(any(target_os = "linux", target_os = "windows"))]
pub mod mistralrs_backend;

use async_trait::async_trait;
use shared::{IpcError, SummaryRequest, SummaryResponse};

#[async_trait]
pub trait LlmSummarizer: Send + Sync {
    async fn summarize(&self, req: SummaryRequest) -> Result<SummaryResponse, IpcError>;
}
