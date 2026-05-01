use leptos::prelude::*;
use shared::{BrailleDevice, Settings};

use crate::components::announcer::AnnouncerHandle;
use crate::ipc;

/// 운영 화면에 항상 노출되는 단말기 전환 위젯.
/// DevicePicker는 첫 진입(braille_device=None)에서만 사용되고, 한 번 선택 후
/// 사용자가 단말기를 바꾸려면 이 컴포넌트로 들어온다 — 가족 공유, 휴대용↔책상용
/// 전환, 단말기 교체 등 현실 사용 시나리오를 모두 커버한다.
#[component]
pub fn DeviceSwitcher(settings: RwSignal<Settings>) -> impl IntoView {
    let active = Signal::derive(move || settings.with(|s| s.braille_device));
    let announcer = use_context::<AnnouncerHandle>();

    let switch = Action::new_local(|d: &BrailleDevice| {
        let device = *d;
        async move {
            let mut next = ipc::get_settings().await.unwrap_or_default();
            next.braille_device = Some(device);
            next.max_braille_cells = device.cells_per_line();
            ipc::save_settings(next.clone()).await?;
            Ok::<_, shared::IpcError>(next)
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(next)) = switch.value().get() {
            settings.set(next.clone());
            if let (Some(h), Some(d)) = (announcer, next.braille_device) {
                h.polite(format!("{}로 전환, {}", d.label_ko(), d.cells_summary_ko()));
            }
        }
    });

    view! {
        <section aria-labelledby="switcher-h">
            <h2 id="switcher-h">"점자단말기"</h2>
            <p role="status" aria-live="polite" aria-label="현재 단말기">
                {move || active.get()
                    .map(|d| format!("현재: {} — {}", d.label_ko(), d.cells_summary_ko()))
                    .unwrap_or_else(|| "선택되지 않음".into())}
            </p>
            <fieldset>
                <legend>"단말기 변경"</legend>
                {BrailleDevice::ALL.iter().map(|d| {
                    let device = *d;
                    let id = format!("switcher-{}", device.slug());
                    let label = format!("{} — {}", device.label_ko(), device.cells_summary_ko());
                    view! {
                        <div>
                            <input
                                type="radio"
                                id=id.clone()
                                name="braille-device-switch"
                                value=device.slug()
                                prop:checked=move || active.get() == Some(device)
                                prop:disabled=move || switch.pending().get()
                                on:change=move |_| { switch.dispatch(device); }
                            />
                            <label for=id>{label}</label>
                        </div>
                    }
                }).collect_view()}
            </fieldset>
        </section>
    }
}
