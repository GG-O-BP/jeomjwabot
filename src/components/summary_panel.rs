use js_sys::Function;
use leptos::prelude::*;
use shared::{EventEnvelope, SummaryRequest};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::ipc;

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

    view! {
        <section aria-labelledby="summary-heading">
            <h2 id="summary-heading">"요약"</h2>
            <p role="status" aria-live="polite">
                {move || format!("{}초마다 요약 갱신", interval_secs.get())}
            </p>
            <p role="status" aria-live="polite" aria-atomic="true" aria-label="최신 요약">
                <Suspense fallback=move || view! { <span>"요약 생성 중 — 잠시 후 다시 출력"</span> }>
                    {move || summary.get().map(|res| match res {
                        Ok(s) => s.text,
                        Err(e) => format!("요약 실패: {e}"),
                    })}
                </Suspense>
            </p>
        </section>
    }
}
