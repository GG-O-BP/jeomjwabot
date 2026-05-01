use leptos::prelude::*;
use shared::{BrailleDevice, Settings};

use crate::components::announcer::AnnouncerHandle;
use crate::ipc;

/// 점자단말기 선택 화면.
/// HTML 네이티브 fieldset/legend로 라디오 그룹 시멘틱을 충족하고, ARIA radiogroup은
/// 중복 announce 방지를 위해 사용하지 않는다. 첫 진입 시 한국에서 가장 흔한 한소네 6를
/// 기본 선택해 두어 점자 사용자가 추가 학습 없이 Tab → Enter로 진행 가능하다.
#[component]
pub fn DevicePicker(settings: RwSignal<Settings>) -> impl IntoView {
    let selected: RwSignal<Option<BrailleDevice>> =
        RwSignal::new(Some(BrailleDevice::BrailleSense6));

    let confirm = Action::new_local(|d: &BrailleDevice| {
        let device = *d;
        async move {
            let mut next = ipc::get_settings().await.unwrap_or_default();
            next.braille_device = Some(device);
            next.max_braille_cells = device.cells_per_line();
            ipc::save_settings(next.clone()).await?;
            Ok::<_, shared::IpcError>(next)
        }
    });

    let announcer = use_context::<AnnouncerHandle>();

    Effect::new(move |_| {
        if let Some(Ok(next)) = confirm.value().get() {
            settings.set(next.clone());
            if let (Some(h), Some(d)) = (announcer, next.braille_device) {
                h.polite(format!("{} 선택, {}", d.label_ko(), d.cells_summary_ko()));
            }
        }
    });

    let on_confirm = move |_| {
        if let Some(d) = selected.get() {
            confirm.dispatch(d);
        }
    };

    view! {
        <section>
            <h1>"사용하시는 점자단말기를 선택해 주세요"</h1>
            <p>"기본 선택은 한소네 6입니다."</p>
            <p>"다른 단말기를 사용 중이라면 화살표 키로 변경해 주세요."</p>

            <fieldset>
                <legend>"점자단말기"</legend>
                {BrailleDevice::ALL.iter().enumerate().map(|(i, d)| {
                    let device = *d;
                    let id = format!("dev-{}", device.slug());
                    let label = format!("{} — {}", device.label_ko(), device.cells_summary_ko());
                    let is_first = i == 0;
                    view! {
                        <div>
                            <input
                                type="radio"
                                id=id.clone()
                                name="braille-device"
                                value=device.slug()
                                autofocus=is_first
                                prop:checked=move || selected.get() == Some(device)
                                on:change=move |_| selected.set(Some(device))
                            />
                            <label for=id>{label}</label>
                        </div>
                    }
                }).collect_view()}
            </fieldset>

            <button
                type="button"
                on:click=on_confirm
                aria-describedby="device-confirm-hint"
                prop:disabled=move || selected.get().is_none() || confirm.pending().get()
            >
                "단말기 선택 확정"
            </button>
            <p id="device-confirm-hint" class="sr-only">
                "단말기를 선택한 뒤 이 단추를 눌러 확정하세요."
            </p>

            <details>
                <summary>"단말기별 연결 안내"</summary>
                {BrailleDevice::ALL.iter().map(|d| {
                    let device = *d;
                    let heading_id = format!("picker-setup-{}", device.slug());
                    view! {
                        <section aria-labelledby=heading_id.clone()>
                            <h2 id=heading_id.clone()>{device.label_ko()}</h2>
                            <ol>
                                {device.setup_steps_ko().iter().map(|step| view! {
                                    <li>{*step}</li>
                                }).collect_view()}
                            </ol>
                        </section>
                    }
                }).collect_view()}
            </details>
        </section>
    }
}
