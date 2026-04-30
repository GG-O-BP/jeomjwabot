use chrono::{DateTime, Utc};
use keyring::Entry;
use shared::{ChzzkSecrets, CimeSecrets, IpcError};

const SERVICE: &str = "jeomjwabot";
const KEY_CHZZK_CLIENT_SECRET: &str = "chzzk.client_secret";
const KEY_CHZZK_ACCESS_TOKEN: &str = "chzzk.access_token";
const KEY_CIME_CLIENT_SECRET: &str = "cime.client_secret";
const KEY_CIME_ACCESS_TOKEN: &str = "cime.access_token";
const KEY_CIME_REFRESH_TOKEN: &str = "cime.refresh_token";
const KEY_CIME_EXPIRES_AT: &str = "cime.expires_at";
const KEY_CIME_SCOPE: &str = "cime.scope";

fn entry(key: &str) -> Result<Entry, IpcError> {
    Entry::new(SERVICE, key).map_err(map_err)
}

fn map_err(e: keyring::Error) -> IpcError {
    IpcError::Internal(format!("keyring: {e}"))
}

fn read_optional(key: &str) -> Result<Option<String>, IpcError> {
    match entry(key)?.get_password() {
        Ok(v) => Ok(Some(v)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(map_err(e)),
    }
}

fn write(key: &str, value: &str) -> Result<(), IpcError> {
    entry(key)?.set_password(value).map_err(map_err)
}

fn delete_if_present(key: &str) -> Result<(), IpcError> {
    match entry(key)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(map_err(e)),
    }
}

fn write_optional(key: &str, value: Option<&str>) -> Result<(), IpcError> {
    match value {
        Some(v) => write(key, v),
        None => delete_if_present(key),
    }
}

pub fn save_chzzk(secrets: &ChzzkSecrets) -> Result<(), IpcError> {
    write(KEY_CHZZK_CLIENT_SECRET, &secrets.client_secret)?;
    match secrets.access_token.as_deref() {
        Some(token) => write(KEY_CHZZK_ACCESS_TOKEN, token)?,
        None => delete_if_present(KEY_CHZZK_ACCESS_TOKEN)?,
    }
    Ok(())
}

pub fn load_chzzk() -> Result<Option<ChzzkSecrets>, IpcError> {
    let Some(client_secret) = read_optional(KEY_CHZZK_CLIENT_SECRET)? else {
        return Ok(None);
    };
    let access_token = read_optional(KEY_CHZZK_ACCESS_TOKEN)?;
    Ok(Some(ChzzkSecrets {
        client_secret,
        access_token,
    }))
}

/// Cime 비밀 전체를 한 번에 저장한다(폼 저장 시).
/// 필드별 부분 갱신은 [`save_cime_client_secret`], [`save_cime_tokens`] 사용.
pub fn save_cime(secrets: &CimeSecrets) -> Result<(), IpcError> {
    write_optional(KEY_CIME_CLIENT_SECRET, secrets.client_secret.as_deref())?;
    write_optional(KEY_CIME_ACCESS_TOKEN, secrets.access_token.as_deref())?;
    write_optional(KEY_CIME_REFRESH_TOKEN, secrets.refresh_token.as_deref())?;
    write_optional(
        KEY_CIME_EXPIRES_AT,
        secrets.expires_at.map(|d| d.to_rfc3339()).as_deref(),
    )?;
    write_optional(KEY_CIME_SCOPE, secrets.scope.as_deref())?;
    Ok(())
}

pub fn load_cime() -> Result<Option<CimeSecrets>, IpcError> {
    let client_secret = read_optional(KEY_CIME_CLIENT_SECRET)?;
    let access_token = read_optional(KEY_CIME_ACCESS_TOKEN)?;
    let refresh_token = read_optional(KEY_CIME_REFRESH_TOKEN)?;
    let scope = read_optional(KEY_CIME_SCOPE)?;
    let expires_at = read_optional(KEY_CIME_EXPIRES_AT)?
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|d| d.with_timezone(&Utc));

    if client_secret.is_none()
        && access_token.is_none()
        && refresh_token.is_none()
        && scope.is_none()
        && expires_at.is_none()
    {
        return Ok(None);
    }
    Ok(Some(CimeSecrets {
        client_secret,
        access_token,
        refresh_token,
        expires_at,
        scope,
    }))
}

/// OAuth 흐름 시작 시 client_secret만 따로 저장(이미 있으면 덮어쓰기).
pub fn save_cime_client_secret(client_secret: &str) -> Result<(), IpcError> {
    write(KEY_CIME_CLIENT_SECRET, client_secret)
}

/// OAuth 흐름 종료 시 토큰 4종을 함께 저장(client_secret은 건드리지 않는다).
pub fn save_cime_tokens(
    access_token: &str,
    refresh_token: Option<&str>,
    expires_at: Option<DateTime<Utc>>,
    scope: Option<&str>,
) -> Result<(), IpcError> {
    write(KEY_CIME_ACCESS_TOKEN, access_token)?;
    write_optional(KEY_CIME_REFRESH_TOKEN, refresh_token)?;
    write_optional(
        KEY_CIME_EXPIRES_AT,
        expires_at.map(|d| d.to_rfc3339()).as_deref(),
    )?;
    write_optional(KEY_CIME_SCOPE, scope)?;
    Ok(())
}

async fn join<T>(
    handle: tauri::async_runtime::JoinHandle<Result<T, IpcError>>,
) -> Result<T, IpcError> {
    handle
        .await
        .map_err(|e| IpcError::Internal(format!("keyring spawn: {e}")))?
}

pub async fn save_chzzk_async(secrets: ChzzkSecrets) -> Result<(), IpcError> {
    join(tauri::async_runtime::spawn_blocking(move || {
        save_chzzk(&secrets)
    }))
    .await
}

pub async fn load_chzzk_async() -> Result<Option<ChzzkSecrets>, IpcError> {
    join(tauri::async_runtime::spawn_blocking(load_chzzk)).await
}

pub async fn save_cime_async(secrets: CimeSecrets) -> Result<(), IpcError> {
    join(tauri::async_runtime::spawn_blocking(move || {
        save_cime(&secrets)
    }))
    .await
}

pub async fn load_cime_async() -> Result<Option<CimeSecrets>, IpcError> {
    join(tauri::async_runtime::spawn_blocking(load_cime)).await
}

pub async fn save_cime_client_secret_async(client_secret: String) -> Result<(), IpcError> {
    join(tauri::async_runtime::spawn_blocking(move || {
        save_cime_client_secret(&client_secret)
    }))
    .await
}

pub async fn save_cime_tokens_async(
    access_token: String,
    refresh_token: Option<String>,
    expires_at: Option<DateTime<Utc>>,
    scope: Option<String>,
) -> Result<(), IpcError> {
    join(tauri::async_runtime::spawn_blocking(move || {
        save_cime_tokens(
            &access_token,
            refresh_token.as_deref(),
            expires_at,
            scope.as_deref(),
        )
    }))
    .await
}
