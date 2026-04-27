use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub chzzk: Option<ChzzkAuth>,
    pub cime: Option<CimeAuth>,
    pub channel_id: String,
    pub summary_interval_secs: u32,
    pub max_braille_cells: u32,
    pub mock_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            chzzk: None,
            cime: None,
            channel_id: String::new(),
            summary_interval_secs: 30,
            max_braille_cells: 32,
            mock_enabled: false,
        }
    }
}

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
