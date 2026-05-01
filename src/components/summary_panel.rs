use js_sys::Function;
use leptos::prelude::*;
use shared::{EventEnvelope, IpcError, Settings, SummaryRequest};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::components::announcer::AnnouncerHandle;
use crate::ipc;

/// 자주 쓰는 요약 주기 프리셋. 점자 사용자가 단축키 한 번으로 선택할 수 있는 단위.
/// 임의 값(예: 45초)은 SettingsForm 의 숫자 입력으로만 가능 — 여기서는 의도적으로 단순화.
const INTERVAL_PRESETS: &[(u32, &str)] = &[
    (10, "10초"),
    (30, "30초"),
    (60, "1분"),
    (180, "3분"),
    (300, "5분"),
];

/// 점자/화면리더 발화용 자연스러운 한국어 표기. 60의 배수는 분, 그 외는 초.
fn humanize_interval(secs: u32) -> String {
    if secs >= 60 && secs % 60 == 0 {
        format!("{}분", secs / 60)
    } else {
        format!("{}초", secs)
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = setInterval)]
    fn js_set_interval(handler: &Function, timeout: i32) -> i32;

    #[wasm_bindgen(js_namespace = window, js_name = clearInterval)]
    fn js_clear_interval(handle: i32);
}

fn schedule_tick(ms: i32, callback: impl FnMut() + 'static) -> i32 {
    let cb = Closure::wrap(Box::new(callback) as Box<dyn FnMut()>);
    let id = js_set_interval(cb.as_ref().unchecked_ref(), ms);
    cb.forget();
    id
}

#[component]
pub fn SummaryPanel(
    events: ReadSignal<Vec<EventEnvelope>>,
    interval_secs: Signal<u32>,
    max_braille_cells: Signal<u32>,
) -> impl IntoView {
    let (tick, set_tick) = signal(0u64);

    Effect::new(move |prev_id: Option<i32>| {
        if let Some(id) = prev_id {
            js_clear_interval(id);
        }
        let secs = interval_secs.get().max(5);
        schedule_tick((secs * 1000) as i32, move || {
            set_tick.update(|n| *n = n.wrapping_add(1));
        })
    });

    let summary = LocalResource::new(move || {
        let _ = tick.get();
        let req = SummaryRequest {
            events: events.get_untracked(),
            max_braille_cells: max_braille_cells.get_untracked(),
        };
        async move { ipc::summarize(req).await }
    });

    // 점자 사용자 1차 원칙: "다음 정상 요약이 도착하기 전엔 이전 출력이 유지돼야 한다."
    // 따라서 성공 응답만 last_good 에 commit 한다. 로딩/에러 상태는 표시 영역을 건드리지 않음.
    // (Effect 13대 원칙 5: 외부 IO(IPC) 결과를 *필터링해서* 영속 signal 에 반영하는 동기화 케이스.
    //  signal-to-signal 단순 파생이 아니므로 Effect 가 정당.)
    let (last_good, set_last_good) = signal(Option::<String>::None);
    Effect::new(move |_| {
        if let Some(Ok(resp)) = summary.get() {
            set_last_good.set(Some(resp.text));
        }
    });

    let display = move || {
        if let Some(text) = last_good.get() {
            // 한 번이라도 정상 요약이 도착했으면 이후 로딩/에러가 와도 이전 텍스트 유지.
            return text;
        }
        // cold start 전용: 아직 첫 정상 요약 미도착일 때만 진행 상태 노출.
        match summary.get() {
            None => "요약 백엔드 준비 중".to_string(),
            Some(Ok(s)) => s.text,
            Some(Err(IpcError::NotReady(msg))) => msg,
            Some(Err(_)) => "요약 시도 중".to_string(),
        }
    };

    view! {
        <section aria-labelledby="summary-heading" id="summary">
            <h2 id="summary-heading">"요약"</h2>
            <p role="status" aria-live="polite" aria-atomic="true" aria-label="최신 요약">
                {display}
            </p>
        </section>
    }
}

/// 운영 화면에 항상 노출되는 요약 주기 전환 위젯.
/// DeviceSwitcher 와 같은 패턴 — 점자 사용자가 자주 쓰는 값을 라디오 하나로 즉시 변경.
/// 임의 값이 필요하면 SettingsForm 의 숫자 입력으로. 변경은 자동 저장(submit 불필요).
#[component]
pub fn SummaryIntervalControl(settings: RwSignal<Settings>) -> impl IntoView {
    let announcer = use_context::<AnnouncerHandle>();
    let current = Signal::derive(move || settings.with(|s| s.summary_interval_secs));

    let save = Action::new_local(|secs: &u32| {
        let secs = *secs;
        async move {
            let mut next = ipc::get_settings().await.unwrap_or_default();
            next.summary_interval_secs = secs.clamp(10, 600);
            ipc::save_settings(next.clone()).await?;
            Ok::<_, IpcError>(next)
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(next)) = save.value().get() {
            settings.set(next.clone());
            if let Some(h) = announcer {
                h.polite(format!(
                    "요약 주기 {}로 변경",
                    humanize_interval(next.summary_interval_secs)
                ));
            }
        }
    });

    view! {
        <section aria-labelledby="interval-switch-h">
            <h2 id="interval-switch-h">"요약 주기"</h2>
            <p role="status" aria-live="polite" aria-label="현재 요약 주기">
                {move || format!("{}마다 요약", humanize_interval(current.get()))}
            </p>
            <fieldset>
                <legend>"주기 변경"</legend>
                {INTERVAL_PRESETS.iter().map(|(secs, label)| {
                    let secs = *secs;
                    let label = (*label).to_string();
                    let id = format!("interval-preset-{secs}");
                    view! {
                        <div>
                            <input
                                type="radio"
                                id=id.clone()
                                name="summary-interval-preset"
                                value=secs.to_string()
                                prop:checked=move || current.get() == secs
                                prop:disabled=move || save.pending().get()
                                on:change=move |_| { save.dispatch(secs); }
                            />
                            <label for=id>{label}</label>
                        </div>
                    }
                }).collect_view()}
            </fieldset>
        </section>
    }
}
