use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Chzzk,
    Cime,
}

impl Platform {
    pub fn label_ko(self) -> &'static str {
        match self {
            Platform::Chzzk => "치지직",
            Platform::Cime => "씨미",
        }
    }
}
