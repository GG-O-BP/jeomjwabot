pub mod mock;

use async_trait::async_trait;
use shared::{IpcError, SummaryRequest, SummaryResponse};

#[async_trait]
pub trait LlmSummarizer: Send + Sync {
    async fn summarize(&self, req: SummaryRequest) -> Result<SummaryResponse, IpcError>;
}
