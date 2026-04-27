use shared::{IpcError, Settings};
use tauri::{AppHandle, Wry};
use tauri_plugin_store::StoreExt;

const STORE_FILE: &str = "settings.json";
const KEY: &str = "settings";

fn store(app: &AppHandle) -> Result<std::sync::Arc<tauri_plugin_store::Store<Wry>>, IpcError> {
    app.store(STORE_FILE)
        .map_err(|e| IpcError::Internal(format!("스토어 열기 실패: {e}")))
}

#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Result<Settings, IpcError> {
    let store = store(&app)?;
    match store.get(KEY) {
        Some(value) => serde_json::from_value(value)
            .map_err(|e| IpcError::Internal(format!("설정 파싱 실패: {e}"))),
        None => Ok(Settings::default()),
    }
}

#[tauri::command]
pub async fn save_settings(app: AppHandle, settings: Settings) -> Result<(), IpcError> {
    let store = store(&app)?;
    let value = serde_json::to_value(&settings).map_err(|e| IpcError::Internal(e.to_string()))?;
    store.set(KEY, value);
    store
        .save()
        .map_err(|e| IpcError::Internal(format!("설정 저장 실패: {e}")))?;
    Ok(())
}

pub async fn load_settings(app: &AppHandle) -> Result<Settings, IpcError> {
    get_settings(app.clone()).await
}
