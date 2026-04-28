use leptos::prelude::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use shared::{
    ChzzkSecrets, CimeSecrets, EventEnvelope, IpcError, Platform, SecretsPresence, Settings,
    SummaryRequest, SummaryResponse,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"], catch)]
    async fn listen(event: &str, handler: &Closure<dyn FnMut(JsValue)>)
        -> Result<JsValue, JsValue>;
}

fn to_args<A: Serialize>(args: &A) -> Result<JsValue, IpcError> {
    serde_wasm_bindgen::to_value(args).map_err(|e| IpcError::Internal(e.to_string()))
}

fn parse_invoke_err(js: JsValue) -> IpcError {
    if js.is_undefined() || js.is_null() {
        return IpcError::Internal("invoke 실패 (사유 미상)".into());
    }
    if let Ok(err) = serde_wasm_bindgen::from_value::<IpcError>(js.clone()) {
        return err;
    }
    if let Some(s) = js.as_string() {
        return IpcError::Internal(s);
    }
    IpcError::Internal(format!("{js:?}"))
}

async fn invoke_typed<A: Serialize, R: DeserializeOwned>(
    cmd: &str,
    args: A,
) -> Result<R, IpcError> {
    let args_js = to_args(&args)?;
    let raw = invoke(cmd, args_js).await.map_err(parse_invoke_err)?;
    serde_wasm_bindgen::from_value(raw).map_err(|e| IpcError::Internal(e.to_string()))
}

async fn invoke_unit<A: Serialize>(cmd: &str, args: A) -> Result<(), IpcError> {
    let args_js = to_args(&args)?;
    invoke(cmd, args_js).await.map_err(parse_invoke_err)?;
    Ok(())
}

#[derive(Serialize)]
struct Empty {}

#[derive(Serialize)]
struct PlatformArgs {
    platform: Platform,
}

#[derive(Serialize)]
struct SettingsArgs {
    settings: Settings,
}

#[derive(Serialize)]
struct SecretsArgs {
    chzzk: Option<ChzzkSecrets>,
    cime: Option<CimeSecrets>,
}

#[derive(Serialize)]
struct SummarizeArgs {
    req: SummaryRequest,
}

pub async fn get_settings() -> Result<Settings, IpcError> {
    invoke_typed("get_settings", Empty {}).await
}

pub async fn save_settings(settings: Settings) -> Result<(), IpcError> {
    invoke_unit("save_settings", SettingsArgs { settings }).await
}

pub async fn save_secrets(
    chzzk: Option<ChzzkSecrets>,
    cime: Option<CimeSecrets>,
) -> Result<(), IpcError> {
    invoke_unit("save_secrets", SecretsArgs { chzzk, cime }).await
}

pub async fn get_secrets_presence() -> Result<SecretsPresence, IpcError> {
    invoke_typed("get_secrets_presence", Empty {}).await
}

pub async fn start_event_source(platform: Platform) -> Result<(), IpcError> {
    invoke_unit("start_event_source", PlatformArgs { platform }).await
}

pub async fn stop_event_source(platform: Platform) -> Result<(), IpcError> {
    invoke_unit("stop_event_source", PlatformArgs { platform }).await
}

pub async fn start_mock_source() -> Result<(), IpcError> {
    invoke_unit("start_mock_source", Empty {}).await
}

pub async fn stop_mock_source() -> Result<(), IpcError> {
    invoke_unit("stop_mock_source", Empty {}).await
}

pub async fn summarize(req: SummaryRequest) -> Result<SummaryResponse, IpcError> {
    invoke_typed("summarize", SummarizeArgs { req }).await
}

#[derive(Deserialize)]
struct TauriEvent<T> {
    payload: T,
}

/// `live-event` 채널을 구독한다. 등록은 앱 생명주기 1회, Closure는 영구.
pub fn on_live_event(mut handler: impl FnMut(EventEnvelope) + 'static) {
    let cb = Closure::wrap(Box::new(move |js: JsValue| {
        match serde_wasm_bindgen::from_value::<TauriEvent<EventEnvelope>>(js) {
            Ok(env) => handler(env.payload),
            Err(e) => leptos::logging::warn!("live-event 파싱 실패: {e}"),
        }
    }) as Box<dyn FnMut(JsValue)>);

    wasm_bindgen_futures::spawn_local(async move {
        if let Err(e) = listen("live-event", &cb).await {
            leptos::logging::warn!("listen 등록 실패: {e:?}");
        }
        cb.forget();
    });
}

/// 백엔드 store에서 설정을 한 번 읽어 signal에 흘려넣는다 (외부 → 내부 동기화).
pub fn hydrate_settings(target: RwSignal<Settings>) {
    wasm_bindgen_futures::spawn_local(async move {
        match get_settings().await {
            Ok(s) => target.set(s),
            Err(e) => leptos::logging::warn!("설정 로딩 실패: {e}"),
        }
    });
}
