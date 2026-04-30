use chrono::Utc;
use shared::{ChzzkAuth, CimeAuth, IpcError, Platform};
use tauri::{AppHandle, State};

use crate::commands::settings::{load_secrets, load_settings};
use crate::oauth::cime as cime_oauth;
use crate::secrets;
use crate::state::{AppState, ConnectionHandle};
use crate::{mock, ws};

/// 만료까지 60초 미만이면 미리 갱신하는 임계값.
const REFRESH_LEEWAY_SECS: i64 = 60;

#[tauri::command]
pub async fn start_event_source(
    state: State<'_, AppState>,
    app: AppHandle,
    platform: Platform,
) -> Result<(), IpcError> {
    let settings = load_settings(&app).await?;
    let (chzzk_secrets, cime_secrets) = load_secrets().await?;
    stop_inner(&state, platform);

    let tx = state.event_tx.clone();
    let handle = match platform {
        Platform::Chzzk => {
            let client_id = settings
                .chzzk_client_id
                .filter(|s| !s.is_empty())
                .ok_or_else(|| IpcError::MissingConfig("치지직 Client ID".into()))?;
            let secrets =
                chzzk_secrets.ok_or_else(|| IpcError::MissingConfig("치지직 인증".into()))?;
            let auth = ChzzkAuth {
                client_id,
                client_secret: secrets.client_secret,
                access_token: secrets.access_token,
            };
            tauri::async_runtime::spawn(async move {
                if let Err(e) = ws::chzzk::run_chzzk(auth, tx).await {
                    tracing::error!(?e, "치지직 세션 종료");
                }
            })
        }
        Platform::Cime => {
            let cime_client_id = settings.cime_client_id.clone();
            let cime_secrets =
                cime_secrets.ok_or_else(|| IpcError::MissingConfig("씨미 인증".into()))?;
            let access_token =
                ensure_fresh_cime_token(cime_client_id.as_deref(), &cime_secrets).await?;
            let auth = CimeAuth { access_token };
            tauri::async_runtime::spawn(async move {
                if let Err(e) = ws::cime::run_cime(auth, tx).await {
                    tracing::error!(?e, "씨미 세션 종료");
                }
            })
        }
    };

    state
        .connections
        .lock()
        .expect("AppState.connections poisoned")
        .insert(platform, ConnectionHandle::new(handle));
    Ok(())
}

/// 저장된 씨미 토큰을 검사해 유효한 access_token을 반환한다.
/// 만료가 60초 안으로 임박했고 client_id/client_secret/refresh_token이 모두 있으면
/// 사용자 개입 없이 자동으로 갱신한 후 새 access_token을 돌려준다.
async fn ensure_fresh_cime_token(
    client_id: Option<&str>,
    secrets_data: &shared::CimeSecrets,
) -> Result<String, IpcError> {
    let access = secrets_data.access_token.clone().ok_or_else(|| {
        IpcError::MissingConfig("씨미 access token이 없습니다. 먼저 계정을 연결하세요.".into())
    })?;

    let needs_refresh = secrets_data
        .expires_at
        .map(|at| (at - Utc::now()).num_seconds() < REFRESH_LEEWAY_SECS)
        .unwrap_or(false);
    if !needs_refresh {
        return Ok(access);
    }

    let (Some(client_id), Some(client_secret), Some(refresh_token)) = (
        client_id.filter(|s| !s.is_empty()),
        secrets_data.client_secret.as_deref(),
        secrets_data.refresh_token.as_deref(),
    ) else {
        tracing::warn!("씨미 토큰 만료 임박이지만 갱신 자격증명이 부족 — 기존 토큰으로 시도");
        return Ok(access);
    };

    tracing::info!("씨미 access token 만료 임박 — 자동 갱신");
    let outcome = cime_oauth::refresh_token(client_id, client_secret, refresh_token).await?;
    secrets::save_cime_tokens_async(
        outcome.access_token.clone(),
        outcome
            .refresh_token
            .clone()
            .or_else(|| Some(refresh_token.to_owned())),
        outcome.expires_at,
        outcome.scope.clone(),
    )
    .await?;
    Ok(outcome.access_token)
}

#[tauri::command]
pub async fn stop_event_source(
    state: State<'_, AppState>,
    platform: Platform,
) -> Result<(), IpcError> {
    stop_inner(&state, platform);
    Ok(())
}

fn stop_inner(state: &AppState, platform: Platform) {
    if let Some(h) = state
        .connections
        .lock()
        .expect("AppState.connections poisoned")
        .remove(&platform)
    {
        h.abort();
    }
}

#[tauri::command]
pub async fn start_mock_source(state: State<'_, AppState>) -> Result<(), IpcError> {
    {
        let mut guard = state
            .mock_handle
            .lock()
            .expect("AppState.mock_handle poisoned");
        if let Some(h) = guard.take() {
            h.abort();
        }
    }
    let tx = state.event_tx.clone();
    let handle = tauri::async_runtime::spawn(async move {
        mock::run_mock(tx).await;
    });
    state
        .mock_handle
        .lock()
        .expect("AppState.mock_handle poisoned")
        .replace(ConnectionHandle::new(handle));
    Ok(())
}

#[tauri::command]
pub async fn stop_mock_source(state: State<'_, AppState>) -> Result<(), IpcError> {
    if let Some(h) = state
        .mock_handle
        .lock()
        .expect("AppState.mock_handle poisoned")
        .take()
    {
        h.abort();
    }
    Ok(())
}
