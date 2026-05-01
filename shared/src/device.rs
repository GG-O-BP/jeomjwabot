use serde::{Deserialize, Serialize};

/// 점자단말기 종류. 셀 폭과 1차 연결 절차가 다르다.
/// 사양 출처는 `docs/devices/{braillesense-6,braille-emotion,dot-pad}.md`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum BrailleDevice {
    BrailleSense6,
    BrailleEmotion40,
    DotPad,
}

impl BrailleDevice {
    pub const ALL: [Self; 3] = [Self::BrailleSense6, Self::BrailleEmotion40, Self::DotPad];

    pub fn cells_per_line(&self) -> u32 {
        match self {
            Self::BrailleSense6 => 32,
            Self::BrailleEmotion40 => 40,
            Self::DotPad => 20,
        }
    }

    pub fn label_ko(&self) -> &'static str {
        match self {
            Self::BrailleSense6 => "한소네 6",
            Self::BrailleEmotion40 => "브레일이모션 40",
            Self::DotPad => "닷패드",
        }
    }

    pub fn cells_summary_ko(&self) -> &'static str {
        match self {
            Self::BrailleSense6 => "한 줄 32셀, 한국어 약 16자",
            Self::BrailleEmotion40 => "한 줄 40셀, 한국어 약 20자",
            Self::DotPad => "텍스트 20셀, 한국어 약 10자",
        }
    }

    pub fn setup_steps_ko(&self) -> &'static [&'static str] {
        match self {
            Self::BrailleSense6 => &[
                "단말기에서 Terminal for Screen Reader 모드 진입",
                "iPhone은 VoiceOver로 Bluetooth 자동 인식",
                "Android는 시스템 Bluetooth 또는 USB-OTG로 자동 인식",
            ],
            Self::BrailleEmotion40 => &[
                "Bluetooth 페어링은 드라이버 필요 없음",
                "USB는 Selvas BLV 드라이버 설치 후 USB-C 연결",
                "VoiceOver 또는 TalkBack이 HIMS 드라이버로 자동 인식",
            ],
            Self::DotPad => &[
                "iPhone iPadOS 15.2 이상에서 VoiceOver 켜기",
                "닷패드를 Bluetooth로 페어링",
                "텍스트 20셀이 자동 출력됩니다",
                "닷패드 그래픽 영역은 다음 업데이트에서 지원",
            ],
        }
    }

    pub fn slug(&self) -> &'static str {
        match self {
            Self::BrailleSense6 => "BrailleSense6",
            Self::BrailleEmotion40 => "BrailleEmotion40",
            Self::DotPad => "DotPad",
        }
    }

    pub fn from_slug(s: &str) -> Option<Self> {
        match s {
            "BrailleSense6" => Some(Self::BrailleSense6),
            "BrailleEmotion40" => Some(Self::BrailleEmotion40),
            "DotPad" => Some(Self::DotPad),
            _ => None,
        }
    }
}
