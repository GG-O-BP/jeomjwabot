use keyring::Entry;
use shared::{ChzzkSecrets, CimeSecrets, IpcError};

const SERVICE: &str = "jeomjwabot";
const KEY_CHZZK_CLIENT_SECRET: &str = "chzzk.client_secret";
const KEY_CHZZK_ACCESS_TOKEN: &str = "chzzk.access_token";
const KEY_CIME_ACCESS_TOKEN: &str = "cime.access_token";

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

pub fn save_cime(secrets: &CimeSecrets) -> Result<(), IpcError> {
    write(KEY_CIME_ACCESS_TOKEN, &secrets.access_token)
}

pub fn load_cime() -> Result<Option<CimeSecrets>, IpcError> {
    Ok(read_optional(KEY_CIME_ACCESS_TOKEN)?.map(|access_token| CimeSecrets { access_token }))
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
