//! 씨미 OAuth 자동화 IPC.
//!
//! 시각장애 사용자 동선을 우선해 설계함:
//! - `start_cime_oauth`: 동기 단계(키링 저장·로컬 포트 바인딩)는 즉시 실행해
//!   포트 충돌 같은 실패를 빨리 알리고, 브라우저 오픈 이후의 비동기 단계는
//!   `oauth-progress` 이벤트로 한 단계씩 한국어 안내를 흘려보낸다.
//! - 콜백 수신 시 점좌봇 창을 자동 포커스해 화면리더가 다음 단계를 즉시 읽게 한다.
//! - `cancel_cime_oauth`로 흐름을 중단할 수 있고, `refresh_cime_token`은
//!   keyring에 저장된 refresh token만으로 사용자 개입 없이 갱신한다.

use std::time::Duration;

use shared::{
    CimeTokenStatus, IpcError, OAuthProgress, OAuthStage, Platform, Settings, CIME_DEFAULT_SCOPES,
    CIME_REDIRECT_URI,
};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::commands::settings::load_settings;
use crate::oauth::{cime, loopback::LoopbackServer};
use crate::secrets;
use crate::state::{AppState, ConnectionHandle};

const CALLBACK_TIMEOUT: Duration = Duration::from_secs(600);

fn emit(app: &AppHandle, stage: OAuthStage, message: impl Into<String>) {
    let payload = OAuthProgress {
        platform: Platform::Cime,
        stage,
        message: message.into(),
    };
    if let Err(e) = app.emit("oauth-progress", &payload) {
        tracing::warn!(?e, "oauth-progress emit 실패");
    }
}

fn focus_main_window(app: &AppHandle) {
    if let Some(window) = app.webview_windows().values().next().cloned() {
        let _ = window.set_focus();
    }
}

fn abort_existing(state: &AppState) {
    if let Some(h) = state
        .oauth_handle
        .lock()
        .expect("AppState.oauth_handle poisoned")
        .take()
    {
        h.abort();
    }
}

fn loopback_addr() -> Result<std::net::SocketAddr, IpcError> {
    let url = url::Url::parse(CIME_REDIRECT_URI)
        .map_err(|e| IpcError::Internal(format!("redirect URI 파싱 실패: {e}")))?;
    let host = url
        .host_str()
        .ok_or_else(|| IpcError::Internal("redirect URI 호스트 누락".into()))?;
    let port = url
        .port()
        .ok_or_else(|| IpcError::Internal("redirect URI 포트 누락".into()))?;
    format!("{host}:{port}")
        .parse()
        .map_err(|e| IpcError::Internal(format!("redirect URI socket 변환 실패: {e}")))
}

#[tauri::command]
pub async fn start_cime_oauth(
    client_id: String,
    client_secret: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), IpcError> {
    let client_id = client_id.trim().to_owned();
    let client_secret = client_secret.trim().to_owned();
    if client_id.is_empty() {
        return Err(IpcError::MissingConfig(
            "씨미 Client ID를 입력해주세요.".into(),
        ));
    }
    if client_secret.is_empty() {
        return Err(IpcError::MissingConfig(
            "씨미 Client Secret을 입력해주세요.".into(),
        ));
    }

    abort_existing(&state);

    // 로컬 콜백 서버를 먼저 바인딩한다. 포트 충돌은 즉시 실패시켜
    // 브라우저를 열기 전에 사용자에게 알린다.
    let addr = loopback_addr()?;
    let server = LoopbackServer::bind(addr)
        .await
        .map_err(|e| IpcError::Internal(format!("로컬 콜백 서버 바인딩 실패: {e}")))?;

    // OAuth 흐름 도중 또는 갱신을 위해 client_secret을 keyring에 보관.
    secrets::save_cime_client_secret_async(client_secret.clone()).await?;

    let csrf = uuid::Uuid::new_v4().to_string();
    let scopes: Vec<&str> = CIME_DEFAULT_SCOPES.to_vec();
    let auth_url = cime::build_auth_url(&client_id, &csrf, &scopes);

    let app_clone = app.clone();
    let handle = tauri::async_runtime::spawn(async move {
        run_flow(app_clone, server, auth_url, csrf, client_id, client_secret).await;
    });

    state
        .oauth_handle
        .lock()
        .expect("AppState.oauth_handle poisoned")
        .replace(ConnectionHandle::new(handle));

    Ok(())
}

async fn run_flow(
    app: AppHandle,
    server: LoopbackServer,
    auth_url: String,
    csrf: String,
    client_id: String,
    client_secret: String,
) {
    emit(
        &app,
        OAuthStage::Starting,
        "씨미 인증을 시작합니다. 시스템 브라우저가 곧 열립니다.",
    );

    if let Err(e) = webbrowser::open(&auth_url) {
        emit(&app, OAuthStage::Error, format!("브라우저 열기 실패: {e}"));
        return;
    }

    emit(
        &app,
        OAuthStage::AwaitingCallback,
        "씨미 인증 페이지를 새 창에서 열었습니다. 브라우저로 이동해 승인 버튼을 눌러주세요. 승인이 끝나면 점좌봇 창으로 자동 전환됩니다.",
    );

    let params = match server.accept_one(CALLBACK_TIMEOUT).await {
        Ok(p) => p,
        Err(e) => {
            emit(
                &app,
                OAuthStage::Error,
                format!("씨미 콜백 대기 실패: {e}. 다시 시도해주세요."),
            );
            return;
        }
    };

    focus_main_window(&app);

    if params.state.as_deref() != Some(csrf.as_str()) {
        emit(
            &app,
            OAuthStage::Error,
            "보안 검증에 실패했습니다(state 불일치). 다시 시도해주세요.",
        );
        return;
    }

    if let Some(err) = params.error.as_deref() {
        let detail = params.error_description.as_deref().unwrap_or("");
        let msg = if detail.is_empty() {
            format!("씨미가 인증을 거부했습니다: {err}")
        } else {
            format!("씨미가 인증을 거부했습니다: {err}. 상세: {detail}")
        };
        emit(&app, OAuthStage::Error, msg);
        return;
    }

    let code = match params.code {
        Some(c) => c,
        None => {
            emit(
                &app,
                OAuthStage::Error,
                "씨미 콜백에 인증 코드가 없습니다. 다시 시도해주세요.",
            );
            return;
        }
    };

    emit(
        &app,
        OAuthStage::Exchanging,
        "인증 코드를 토큰으로 교환하는 중입니다.",
    );

    let outcome = match cime::exchange_code(&client_id, &client_secret, &code).await {
        Ok(o) => o,
        Err(e) => {
            emit(&app, OAuthStage::Error, format!("토큰 교환 실패: {e}"));
            return;
        }
    };

    emit(
        &app,
        OAuthStage::Saving,
        "발급된 토큰을 안전한 자격 증명 저장소에 보관 중입니다.",
    );

    if let Err(e) = secrets::save_cime_tokens_async(
        outcome.access_token.clone(),
        outcome.refresh_token.clone(),
        outcome.expires_at,
        outcome.scope.clone(),
    )
    .await
    {
        emit(&app, OAuthStage::Error, format!("토큰 저장 실패: {e}"));
        return;
    }

    let expiry_msg = match outcome.expires_at {
        Some(at) => format!(" 토큰 만료는 {} 입니다.", at.to_rfc3339()),
        None => String::new(),
    };
    emit(
        &app,
        OAuthStage::Saved,
        format!("씨미 계정이 연결되었습니다.{expiry_msg}"),
    );
}

#[tauri::command]
pub async fn cancel_cime_oauth(state: State<'_, AppState>, app: AppHandle) -> Result<(), IpcError> {
    let had_handle = {
        let mut guard = state
            .oauth_handle
            .lock()
            .expect("AppState.oauth_handle poisoned");
        if let Some(h) = guard.take() {
            h.abort();
            true
        } else {
            false
        }
    };
    if had_handle {
        emit(&app, OAuthStage::Cancelled, "씨미 인증을 취소했습니다.");
    }
    Ok(())
}

#[tauri::command]
pub async fn refresh_cime_token(app: AppHandle) -> Result<(), IpcError> {
    let settings: Settings = load_settings(&app).await?;
    let client_id = settings
        .cime_client_id
        .filter(|s| !s.is_empty())
        .ok_or_else(|| IpcError::MissingConfig("씨미 Client ID가 비어있습니다.".into()))?;

    let stored = secrets::load_cime_async()
        .await?
        .ok_or_else(|| IpcError::MissingConfig("저장된 씨미 자격 증명이 없습니다.".into()))?;
    let client_secret = stored.client_secret.ok_or_else(|| {
        IpcError::MissingConfig("씨미 Client Secret이 저장되어 있지 않습니다.".into())
    })?;
    let refresh_token = stored.refresh_token.ok_or_else(|| {
        IpcError::MissingConfig("씨미 Refresh Token이 없습니다. 다시 연결해주세요.".into())
    })?;

    emit(
        &app,
        OAuthStage::Exchanging,
        "씨미 토큰을 갱신하는 중입니다.",
    );

    let outcome = cime::refresh_token(&client_id, &client_secret, &refresh_token).await?;

    secrets::save_cime_tokens_async(
        outcome.access_token,
        outcome.refresh_token.or(Some(refresh_token)),
        outcome.expires_at,
        outcome.scope,
    )
    .await?;

    let expiry_msg = match outcome.expires_at {
        Some(at) => format!(" 새 만료 시각: {}", at.to_rfc3339()),
        None => String::new(),
    };
    emit(
        &app,
        OAuthStage::Saved,
        format!("씨미 토큰이 갱신되었습니다.{expiry_msg}"),
    );
    Ok(())
}

#[tauri::command]
pub async fn get_cime_token_status() -> Result<CimeTokenStatus, IpcError> {
    let stored = secrets::load_cime_async().await?;
    Ok(match stored {
        Some(s) => CimeTokenStatus {
            access_token_present: s.access_token.is_some(),
            client_secret_present: s.client_secret.is_some(),
            expires_at: s.expires_at,
            scope: s.scope,
        },
        None => CimeTokenStatus {
            access_token_present: false,
            client_secret_present: false,
            expires_at: None,
            scope: None,
        },
    })
}
