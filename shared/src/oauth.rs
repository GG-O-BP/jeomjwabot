use serde::{Deserialize, Serialize};

use crate::Platform;

/// OAuth 흐름의 단계. 화면리더 사용자를 위해 UI는 각 단계마다 명확한 한국어 문장을 announce한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OAuthStage {
    /// 흐름 시작 — 로컬 콜백 서버 바인딩 직전.
    Starting,
    /// 시스템 브라우저로 인증 페이지 오픈, 사용자 승인 대기.
    AwaitingCallback,
    /// 콜백 코드 수신 후 토큰 엔드포인트로 교환 중.
    Exchanging,
    /// keyring에 토큰 저장 중.
    Saving,
    /// 완료.
    Saved,
    /// 사용자가 취소.
    Cancelled,
    /// 오류 발생.
    Error,
}

/// 백엔드가 oauth-progress 이벤트로 흘려보내는 페이로드.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProgress {
    pub platform: Platform,
    pub stage: OAuthStage,
    /// 화면리더가 그대로 읽을 한국어 문장. 이모지·영문 약어 금지.
    pub message: String,
}

/// 점좌봇이 씨미에게 요청하는 표준 scope. 채팅·후원·구독 이벤트 수신 + 채팅 송신.
pub const CIME_DEFAULT_SCOPES: &[&str] = &[
    "READ:LIVE_CHAT",
    "READ:DONATION",
    "READ:SUBSCRIPTION",
    "WRITE:LIVE_CHAT",
];

/// 점좌봇이 OAuth 콜백 수신에 사용하는 고정 redirect URI.
/// 사용자는 씨미 개발자 포털에서 자기 앱의 Redirect URI로 이 값을 등록해야 한다.
pub const CIME_REDIRECT_URI: &str = "http://127.0.0.1:8765/callback";
