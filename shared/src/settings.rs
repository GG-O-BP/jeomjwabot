use serde::{Deserialize, Serialize};

/// 비비밀 설정만 보관. 비밀(client_secret, access_token)은 OS keyring에 저장.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub chzzk_client_id: Option<String>,
    pub channel_id: String,
    pub summary_interval_secs: u32,
    pub max_braille_cells: u32,
    pub mock_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            chzzk_client_id: None,
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

/// keyring에 저장되는 씨미 비밀.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CimeSecrets {
    pub access_token: String,
}

/// 폼 placeholder/aria 안내용. 평문은 절대 노출하지 않고 존재 여부만 전달.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsPresence {
    pub chzzk_present: bool,
    pub cime_present: bool,
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
