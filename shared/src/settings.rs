use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 비비밀 설정만 보관. 비밀(client_secret, access_token, refresh_token 등)은 OS keyring에 저장.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub chzzk_client_id: Option<String>,
    pub cime_client_id: Option<String>,
    pub channel_id: String,
    pub summary_interval_secs: u32,
    pub max_braille_cells: u32,
    pub mock_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            chzzk_client_id: None,
            cime_client_id: None,
            channel_id: String::new(),
            summary_interval_secs: 30,
            max_braille_cells: 32,
            mock_enabled: false,
        }
    }
}

/// keyring에 저장되는 치지직 비밀. 평문으로 store/JSON에 직렬화 금지.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChzzkSecrets {
    pub client_secret: String,
    pub access_token: Option<String>,
}

/// keyring에 저장되는 씨미 비밀. OAuth 자격증명(client_secret)과
/// 발급된 토큰(access_token/refresh_token + 만료 시점)을 함께 보관.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CimeSecrets {
    pub client_secret: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub scope: Option<String>,
}

/// 폼 placeholder/aria 안내용. 평문은 절대 노출하지 않고 존재 여부만 전달.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsPresence {
    pub chzzk_present: bool,
    pub cime_present: bool,
}

/// 씨미 토큰의 자세한 상태. UI가 만료 시점·스코프·OAuth 가능 여부를 표시할 수 있게 한다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CimeTokenStatus {
    pub access_token_present: bool,
    pub client_secret_present: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub scope: Option<String>,
}

/// WS/HTTP 호출에 쓰이는 런타임 합성 타입. Settings + Secrets에서 합성된다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChzzkAuth {
    pub client_id: String,
    pub client_secret: String,
    pub access_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CimeAuth {
    pub access_token: String,
}
