use shared::{IpcError, SummaryRequest, SummaryResponse};
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn summarize(
    req: SummaryRequest,
    state: State<'_, AppState>,
) -> Result<SummaryResponse, IpcError> {
    if let Some(active) = state.summarizer.get() {
        return active.summarize(req).await;
    }
    fallback(req).await
}

/// 데스크톱(Linux/Windows): Qwen3.6 모델이 로드 전이거나 실패한 상태.
/// 점자단말기로 가짜 요약을 흘려보내지 않도록 명시적으로 실패시킨다.
#[cfg(any(target_os = "linux", target_os = "windows"))]
async fn fallback(_req: SummaryRequest) -> Result<SummaryResponse, IpcError> {
    Err(IpcError::Internal(
        "요약 모델이 아직 준비되지 않았습니다. 모델 다운로드 또는 로드 상태를 확인하세요.".into(),
    ))
}

/// 모바일(iOS/Android) 및 그 외: native LLM 브릿지 도입 전까지 mock 사용.
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
async fn fallback(req: SummaryRequest) -> Result<SummaryResponse, IpcError> {
    use crate::llm::mock::MockSummarizer;
    MockSummarizer.summarize(req).await
}
