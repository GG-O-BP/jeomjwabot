use shared::{IpcError, SummaryRequest, SummaryResponse};

use crate::llm::{mock::MockSummarizer, LlmSummarizer};

#[tauri::command]
pub async fn summarize(req: SummaryRequest) -> Result<SummaryResponse, IpcError> {
    MockSummarizer.summarize(req).await
}
