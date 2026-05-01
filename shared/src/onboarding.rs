use crate::{CimeTokenStatus, SecretsPresence, Settings};

/// 신규 사용자 진입 동선의 분기 상태. UI 라우터가 이 값으로 화면을 결정한다.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum OnboardingState {
    /// 점자단말기 미선택 — DevicePicker 화면.
    NeedsDevice,
    /// 단말기는 선택됐지만 채널·인증 중 하나라도 비어 있음 — Welcome/Wizard 화면.
    NeedsConfig,
    /// 채널 + (치지직 또는 씨미) 인증 완료 — RuntimeView.
    Configured,
}

pub fn compute(
    settings: &Settings,
    presence: &SecretsPresence,
    token: &CimeTokenStatus,
) -> OnboardingState {
    if settings.braille_device.is_none() {
        return OnboardingState::NeedsDevice;
    }
    // mock 시연 중에도 운영 화면(RuntimeView) 마운트가 필요하다.
    // 점자 사용자가 토큰 등록 부담 없이 첫 점자 출력까지 도달하는 동선.
    if settings.mock_enabled {
        return OnboardingState::Configured;
    }
    let has_channel = !settings.channel_id.trim().is_empty();
    let has_chzzk = settings
        .chzzk_client_id
        .as_deref()
        .map(|s| !s.is_empty())
        .unwrap_or(false)
        && presence.chzzk_present;
    let has_cime = token.access_token_present;
    if has_channel && (has_chzzk || has_cime) {
        OnboardingState::Configured
    } else {
        OnboardingState::NeedsConfig
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BrailleDevice;

    fn empty_presence() -> SecretsPresence {
        SecretsPresence {
            chzzk_present: false,
            cime_present: false,
        }
    }

    fn empty_token() -> CimeTokenStatus {
        CimeTokenStatus {
            access_token_present: false,
            client_secret_present: false,
            expires_at: None,
            scope: None,
        }
    }

    #[test]
    fn needs_device_when_no_device() {
        let s = Settings::default();
        assert_eq!(
            compute(&s, &empty_presence(), &empty_token()),
            OnboardingState::NeedsDevice
        );
    }

    #[test]
    fn needs_config_when_device_only() {
        let s = Settings {
            braille_device: Some(BrailleDevice::BrailleSense6),
            ..Default::default()
        };
        assert_eq!(
            compute(&s, &empty_presence(), &empty_token()),
            OnboardingState::NeedsConfig
        );
    }

    #[test]
    fn configured_when_channel_and_cime_token() {
        let s = Settings {
            braille_device: Some(BrailleDevice::BrailleSense6),
            channel_id: "abc".into(),
            ..Default::default()
        };
        let t = CimeTokenStatus {
            access_token_present: true,
            ..empty_token()
        };
        assert_eq!(
            compute(&s, &empty_presence(), &t),
            OnboardingState::Configured
        );
    }

    #[test]
    fn configured_when_mock_enabled_alone() {
        let s = Settings {
            braille_device: Some(BrailleDevice::BrailleSense6),
            mock_enabled: true,
            ..Default::default()
        };
        assert_eq!(
            compute(&s, &empty_presence(), &empty_token()),
            OnboardingState::Configured
        );
    }

    #[test]
    fn configured_when_channel_and_chzzk() {
        let s = Settings {
            braille_device: Some(BrailleDevice::DotPad),
            channel_id: "abc".into(),
            chzzk_client_id: Some("cid".into()),
            ..Default::default()
        };
        let p = SecretsPresence {
            chzzk_present: true,
            ..empty_presence()
        };
        assert_eq!(compute(&s, &p, &empty_token()), OnboardingState::Configured);
    }
}
