use leptos::prelude::*;
use shared::{EventEnvelope, LiveEvent, Platform, SystemKind};

#[component]
pub fn ConnectionStatus(
    events: ReadSignal<Vec<EventEnvelope>>,
    mock_enabled: Signal<bool>,
) -> impl IntoView {
    let last_kinds = move || {
        let mut chzzk: Option<SystemKind> = None;
        let mut cime: Option<SystemKind> = None;
        events.with(|evs| {
            for env in evs.iter().rev() {
                if let LiveEvent::System(sys) = &env.payload {
                    let target = match env.platform {
                        Platform::Chzzk => &mut chzzk,
                        Platform::Cime => &mut cime,
                    };
                    if target.is_none() {
                        *target = Some(sys.kind);
                    }
                    if chzzk.is_some() && cime.is_some() {
                        break;
                    }
                }
            }
        });
        (chzzk, cime)
    };

    let polite_text = move || {
        let (chzzk, cime) = last_kinds();
        let label = |name: &str, kind: Option<SystemKind>| match kind {
            Some(SystemKind::Connected) | Some(SystemKind::Subscribed) => format!("{name} 연결됨"),
            Some(SystemKind::Unsubscribed) => format!("{name} 구독 해제"),
            Some(SystemKind::Disconnected) => format!("{name} 연결 끊김"),
            Some(SystemKind::Revoked) => format!("{name} 권한 회수됨"),
            None => format!("{name} 대기"),
        };
        let mock = if mock_enabled.get() {
            " · 모의 데이터 사용 중"
        } else {
            ""
        };
        format!(
            "{} · {}{}",
            label("치지직", chzzk),
            label("씨미", cime),
            mock
        )
    };

    let urgent_text = move || {
        let (chzzk, cime) = last_kinds();
        let urgent = |name: &str, kind: Option<SystemKind>| match kind {
            Some(SystemKind::Revoked) => {
                Some(format!("{name} 권한이 회수되었습니다 — 다시 로그인 필요"))
            }
            Some(SystemKind::Disconnected) => Some(format!("{name} 연결이 끊겼습니다")),
            _ => None,
        };
        match (urgent("치지직", chzzk), urgent("씨미", cime)) {
            (Some(a), Some(b)) => format!("{a} / {b}"),
            (Some(a), None) | (None, Some(a)) => a,
            (None, None) => String::new(),
        }
    };

    view! {
        <p role="status" aria-live="polite" aria-label="플랫폼 연결 상태">
            {polite_text}
        </p>
        <p role="alert" aria-live="assertive" aria-atomic="true">
            {urgent_text}
        </p>
    }
}
