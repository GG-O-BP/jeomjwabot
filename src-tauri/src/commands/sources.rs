use shared::{IpcError, Platform};
use tauri::{AppHandle, State};

use crate::commands::settings::load_settings;
use crate::state::{AppState, ConnectionHandle};
use crate::{mock, ws};

#[tauri::command]
pub async fn start_event_source(
    state: State<'_, AppState>,
    app: AppHandle,
    platform: Platform,
) -> Result<(), IpcError> {
    let settings = load_settings(&app).await?;
    stop_inner(&state, platform);

    let tx = state.event_tx.clone();
    let handle = match platform {
        Platform::Chzzk => {
            let auth = settings
                .chzzk
                .ok_or_else(|| IpcError::MissingConfig("치지직 인증".into()))?;
            tauri::async_runtime::spawn(async move {
                if let Err(e) = ws::chzzk::run_chzzk(auth, tx).await {
                    tracing::error!(?e, "치지직 세션 종료");
                }
            })
        }
        Platform::Cime => {
            let auth = settings
                .cime
                .ok_or_else(|| IpcError::MissingConfig("씨미 인증".into()))?;
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
