use leptos::prelude::*;
use shared::{EventEnvelope, Settings};

use crate::components::announcer::AnnouncerHandle;
use crate::components::connection_status::ConnectionStatus;
use crate::components::device_switcher::DeviceSwitcher;
use crate::components::event_log::EventLog;
use crate::components::settings::SettingsForm;
use crate::components::summary_panel::{SummaryIntervalControl, SummaryPanel};

/// 단말기·채널·인증이 모두 준비된 상태의 운영 화면.
/// WCAG 2.4.1 Bypass Blocks를 충족하는 스킵 링크가 첫 포커스를 받는다.
#[component]
pub fn RuntimeView(
    events: ReadSignal<Vec<EventEnvelope>>,
    settings: RwSignal<Settings>,
) -> impl IntoView {
    let interval_secs = Signal::derive(move || settings.with(|s| s.summary_interval_secs));
    let max_cells = Signal::derive(move || settings.with(|s| s.max_braille_cells));
    let mock_enabled = Signal::derive(move || settings.with(|s| s.mock_enabled));

    // 진입 시 1회 활성 단말기를 polite로 발화 — 사용자가 매 부팅마다 단말기 확인.
    let announcer = use_context::<AnnouncerHandle>();
    Effect::new(move |prev: Option<()>| {
        if prev.is_none() {
            if let (Some(h), Some(d)) = (announcer, settings.with(|s| s.braille_device)) {
                h.polite(format!(
                    "현재 단말기 {}, {}",
                    d.label_ko(),
                    d.cells_summary_ko()
                ));
            }
        }
    });

    view! {
        <a href="#summary" class="sr-only-focusable" autofocus>
            "요약으로 건너뛰기"
        </a>

        <DeviceSwitcher settings=settings />

        <SummaryIntervalControl settings=settings />

        <section aria-labelledby="status-h">
            <h2 id="status-h">"상태"</h2>
            <ConnectionStatus events=events mock_enabled=mock_enabled />
        </section>

        <SummaryPanel
            events=events
            interval_secs=interval_secs
            max_braille_cells=max_cells
        />

        <details>
            <summary>"자세히 (시각 전용)"</summary>
            <EventLog events=events />
        </details>

        <details>
            <summary>"설정 변경"</summary>
            <SettingsForm settings=settings />
        </details>
    }
}
