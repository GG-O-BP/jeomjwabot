use leptos::prelude::*;
use shared::{
    compute_onboarding, CimeTokenStatus, EventEnvelope, OAuthStage, OnboardingState,
    SecretsPresence, Settings,
};

use crate::components::announcer::LiveAnnouncer;
use crate::components::device_picker::DevicePicker;
use crate::components::onboarding::OnboardingFlow;
use crate::components::runtime_view::RuntimeView;
use crate::ipc;

const EVENT_BUFFER_LIMIT: usize = 200;

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

#[component]
pub fn App() -> impl IntoView {
    let events: RwSignal<Vec<EventEnvelope>> = RwSignal::new(Vec::new());
    let settings: RwSignal<Settings> = RwSignal::new(Settings::default());
    let presence: RwSignal<SecretsPresence> = RwSignal::new(empty_presence());
    let token_status: RwSignal<CimeTokenStatus> = RwSignal::new(empty_token());
    let hydrated: RwSignal<bool> = RwSignal::new(false);

    ipc::on_live_event(move |env| {
        events.update(|v| {
            v.push(env);
            if v.len() > EVENT_BUFFER_LIMIT {
                let drop_count = v.len() - EVENT_BUFFER_LIMIT;
                v.drain(0..drop_count);
            }
        });
    });

    // OAuth 완료 시 keyring/token 상태가 바뀌므로 외부 signal을 다시 가져온다.
    ipc::on_oauth_progress(move |p| {
        if matches!(p.stage, OAuthStage::Saved) {
            ipc::hydrate_presence(presence);
            ipc::hydrate_cime_token_status(token_status);
        }
    });

    ipc::hydrate_all(settings, presence, token_status, hydrated);

    let onboarding = move || {
        settings.with(|s| presence.with(|p| token_status.with(|t| compute_onboarding(s, p, t))))
    };

    view! {
        <main class="container" tabindex="-1">
            <h1>"점좌봇"</h1>
            <LiveAnnouncer />
            <Show
                when=move || hydrated.get()
                fallback=|| view! {
                    <p role="status" aria-live="polite">"앱 준비 중"</p>
                }
            >
                {move || match onboarding() {
                    OnboardingState::NeedsDevice => view! {
                        <DevicePicker settings=settings />
                    }.into_any(),
                    OnboardingState::NeedsConfig => view! {
                        <OnboardingFlow settings=settings />
                    }.into_any(),
                    OnboardingState::Configured => view! {
                        <RuntimeView events=events.read_only() settings=settings />
                    }.into_any(),
                }}
            </Show>
        </main>
    }
}
