use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(tag = "kind", content = "message", rename_all = "snake_case")]
pub enum IpcError {
    #[error("인증 실패: {0}")]
    Auth(String),
    #[error("네트워크 오류: {0}")]
    Network(String),
    #[error("프로토콜 오류: {0}")]
    Protocol(String),
    #[error("설정이 누락되었습니다: {0}")]
    MissingConfig(String),
    /// 일시적 미준비 상태(LLM 로딩 중 등). 사용자에게는 "준비 중"으로 안내하고
    /// 다음 폴링/tick에 자동 재시도되는 흐름이라 영구 실패와 구분한다.
    #[error("준비 중: {0}")]
    NotReady(String),
    #[error("내부 오류: {0}")]
    Internal(String),
}
