use leptos::prelude::*;
use shared::{EventEnvelope, Settings};

use crate::components::connection_status::ConnectionStatus;
use crate::components::event_log::EventLog;
use crate::components::settings::SettingsForm;
use crate::components::summary_panel::SummaryPanel;
use crate::ipc;

const EVENT_BUFFER_LIMIT: usize = 200;

#[component]
pub fn App() -> impl IntoView {
    let events: RwSignal<Vec<EventEnvelope>> = RwSignal::new(Vec::new());
    let settings: RwSignal<Settings> = RwSignal::new(Settings::default());

    ipc::on_live_event(move |env| {
        events.update(|v| {
            v.push(env);
            if v.len() > EVENT_BUFFER_LIMIT {
                let drop_count = v.len() - EVENT_BUFFER_LIMIT;
                v.drain(0..drop_count);
            }
        });
    });

    ipc::hydrate_settings(settings);

    let interval_secs = Signal::derive(move || settings.with(|s| s.summary_interval_secs));
    let max_cells = Signal::derive(move || settings.with(|s| s.max_braille_cells));
    let mock_enabled = Signal::derive(move || settings.with(|s| s.mock_enabled));

    view! {
        <main class="container">
            <h1>"점자봇"</h1>
            <p class="lede">
                "라이브 방송 채팅·후원·구독을 점자단말기로 따라잡습니다."
            </p>
            <ConnectionStatus events=events.read_only() mock_enabled=mock_enabled />
            <SettingsForm settings=settings />
            <EventLog events=events.read_only() />
            <SummaryPanel
                events=events.read_only()
                interval_secs=interval_secs
                max_braille_cells=max_cells
            />
        </main>
    }
}
