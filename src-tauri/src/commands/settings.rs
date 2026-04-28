use serde_json::Value;
use shared::{ChzzkSecrets, CimeSecrets, IpcError, SecretsPresence, Settings};
use tauri::{AppHandle, Wry};
use tauri_plugin_store::StoreExt;

use crate::secrets;

const STORE_FILE: &str = "settings.json";
const KEY: &str = "settings";

fn store(app: &AppHandle) -> Result<std::sync::Arc<tauri_plugin_store::Store<Wry>>, IpcError> {
    app.store(STORE_FILE)
        .map_err(|e| IpcError::Internal(format!("스토어 열기 실패: {e}")))
}

#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Result<Settings, IpcError> {
    let store = store(&app)?;
    let raw = match store.get(KEY) {
        Some(v) => v,
        None => return Ok(Settings::default()),
    };

    let migrated = migrate_legacy_secrets(raw).await?;
    if let Some(new_value) = migrated.write_back {
        store.set(KEY, new_value);
        store
            .save()
            .map_err(|e| IpcError::Internal(format!("설정 저장 실패: {e}")))?;
    }

    serde_json::from_value(migrated.settings)
        .map_err(|e| IpcError::Internal(format!("설정 파싱 실패: {e}")))
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

#[tauri::command]
pub async fn save_secrets(
    chzzk: Option<ChzzkSecrets>,
    cime: Option<CimeSecrets>,
) -> Result<(), IpcError> {
    if let Some(s) = chzzk {
        secrets::save_chzzk_async(s).await?;
    }
    if let Some(s) = cime {
        secrets::save_cime_async(s).await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_secrets_presence() -> Result<SecretsPresence, IpcError> {
    let chzzk_present = secrets::load_chzzk_async().await?.is_some();
    let cime_present = secrets::load_cime_async().await?.is_some();
    Ok(SecretsPresence {
        chzzk_present,
        cime_present,
    })
}

pub async fn load_settings(app: &AppHandle) -> Result<Settings, IpcError> {
    get_settings(app.clone()).await
}

pub async fn load_secrets() -> Result<(Option<ChzzkSecrets>, Option<CimeSecrets>), IpcError> {
    Ok((
        secrets::load_chzzk_async().await?,
        secrets::load_cime_async().await?,
    ))
}

struct Migrated {
    settings: Value,
    write_back: Option<Value>,
}

/// 구 형식(`chzzk: {client_id, client_secret, access_token}`, `cime: {access_token}`)에서
/// 비밀 필드를 keyring으로 옮기고 store에는 비비밀만 남긴다. idempotent.
async fn migrate_legacy_secrets(raw: Value) -> Result<Migrated, IpcError> {
    let Value::Object(mut map) = raw else {
        return Ok(Migrated {
            settings: Value::Object(serde_json::Map::new()),
            write_back: None,
        });
    };
    let mut changed = false;

    if let Some(Value::Object(chzzk)) = map.remove("chzzk") {
        let client_id = chzzk
            .get("client_id")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let client_secret = chzzk
            .get("client_secret")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let access_token = chzzk
            .get("access_token")
            .and_then(Value::as_str)
            .map(str::to_owned);

        if let Some(secret) = client_secret {
            secrets::save_chzzk_async(ChzzkSecrets {
                client_secret: secret,
                access_token,
            })
            .await?;
            changed = true;
        }
        if let Some(id) = client_id {
            map.insert("chzzk_client_id".into(), Value::String(id));
            changed = true;
        }
    }

    if let Some(Value::Object(cime)) = map.remove("cime") {
        if let Some(token) = cime.get("access_token").and_then(Value::as_str) {
            secrets::save_cime_async(CimeSecrets {
                access_token: token.to_owned(),
            })
            .await?;
            changed = true;
        }
    }

    let settings = Value::Object(map);
    Ok(Migrated {
        write_back: changed.then(|| settings.clone()),
        settings,
    })
}
