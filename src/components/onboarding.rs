use leptos::prelude::*;
use shared::{BrailleDevice, Settings};

use crate::components::announcer::AnnouncerHandle;
use crate::components::settings::SettingsForm;
use crate::ipc;

/// 단말기 선택은 끝났지만 채널·인증이 미완인 상태에서 보이는 진입 화면.
/// 두 갈래: (1) 모의 데이터 시연으로 즉시 점자 동선 체험, (2) 실 계정 연결.
#[component]
pub fn OnboardingFlow(settings: RwSignal<Settings>) -> impl IntoView {
    let show_setup = RwSignal::new(false);

    view! {
        <Show
            when=move || !show_setup.get()
            fallback=move || view! { <SetupSection settings=settings /> }
        >
            <WelcomePanel settings=settings show_setup=show_setup />
        </Show>
    }
}

#[component]
fn WelcomePanel(settings: RwSignal<Settings>, show_setup: RwSignal<bool>) -> impl IntoView {
    let device = Signal::derive(move || settings.with(|s| s.braille_device));
    let cells_summary = move || {
        device
            .get()
            .map(|d| d.cells_summary_ko().to_string())
            .unwrap_or_default()
    };

    let announcer = use_context::<AnnouncerHandle>();

    let start_demo = Action::new_local(|_: &()| async move {
        let mut next = ipc::get_settings().await.unwrap_or_default();
        next.mock_enabled = true;
        ipc::save_settings(next.clone()).await?;
        ipc::start_mock_source().await?;
        Ok::<_, shared::IpcError>(next)
    });

    Effect::new(move |_| {
        if let Some(Ok(next)) = start_demo.value().get() {
            settings.set(next);
            if let Some(h) = announcer {
                h.polite("모의 채팅 수신 중. 약 30초 뒤 첫 요약.");
            }
        }
    });

    view! {
        <section aria-labelledby="welcome-h" tabindex="-1">
            <h1 id="welcome-h">"환영합니다"</h1>
            <p>"라이브 채팅을 점자로 받습니다."</p>
            <p>{move || format!("{}로 출력합니다.", cells_summary())}</p>

            <button
                type="button"
                autofocus
                on:click=move |_| { start_demo.dispatch(()); }
                prop:disabled=move || start_demo.pending().get()
            >
                "모의 데이터 시연 시작"
            </button>

            <button type="button" on:click=move |_| show_setup.set(true)>
                "실 계정 연결"
            </button>

            <details>
                <summary>"점자 출력은 어떻게 동작하나요"</summary>
                {move || device.get().map(|d| view! {
                    <section>
                        <h2>{d.label_ko()}</h2>
                        <ol>
                            {d.setup_steps_ko().iter().map(|step| view! {
                                <li>{*step}</li>
                            }).collect_view()}
                        </ol>
                    </section>
                })}
            </details>
        </section>
    }
}

/// 실 계정 연결을 선택했을 때 노출되는 설정 영역.
/// SettingsForm을 그대로 마운트해 채널/인증 모두 같은 컴포넌트에서 처리한다.
#[component]
fn SetupSection(settings: RwSignal<Settings>) -> impl IntoView {
    view! {
        <section aria-labelledby="setup-h" tabindex="-1">
            <h1 id="setup-h">"실 계정 연결"</h1>
            <p>"채널 ID와 치지직 또는 씨미 자격증명을 입력해 주세요."</p>
            <SettingsForm settings=settings />
        </section>
    }
}

// 미사용 경고를 막기 위해 BrailleDevice를 explicitly use (문서화 목적).
#[allow(dead_code)]
fn _device_used(_: BrailleDevice) {}
