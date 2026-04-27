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
    #[error("내부 오류: {0}")]
    Internal(String),
}
