mod auth;
mod commands;
mod llm;
mod mock;
mod oauth;
mod secrets;
mod state;
mod ws;

use tauri::{Emitter, Manager};

use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,jeomjwabot_lib=debug".into()),
        )
        .try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(AppState::new())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let mut rx = app.state::<AppState>().event_tx.subscribe();
            tauri::async_runtime::spawn(async move {
                while let Ok(env) = rx.recv().await {
                    if let Err(e) = app_handle.emit("live-event", &env) {
                        tracing::warn!(?e, "live-event emit 실패");
                    }
                }
            });

            // mock 시연은 일회성. 이전 세션에서 켠 채로 종료했더라도 다음 부팅엔
            // 신규 진입 동선(NeedsConfig 또는 NeedsDevice)으로 다시 시작한다.
            spawn_reset_mock_enabled(app.handle().clone());

            #[cfg(any(target_os = "linux", target_os = "windows"))]
            spawn_desktop_llm_loader(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::save_secrets,
            commands::settings::get_secrets_presence,
            commands::sources::start_event_source,
            commands::sources::stop_event_source,
            commands::sources::start_mock_source,
            commands::sources::stop_mock_source,
            commands::llm::summarize,
            commands::oauth::start_cime_oauth,
            commands::oauth::cancel_cime_oauth,
            commands::oauth::refresh_cime_token,
            commands::oauth::get_cime_token_status,
        ])
        .run(tauri::generate_context!())
        .expect("점좌봇 실행 실패");
}

fn spawn_reset_mock_enabled(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        match commands::settings::load_settings(&app).await {
            Ok(mut s) if s.mock_enabled => {
                s.mock_enabled = false;
                if let Err(e) = commands::settings::save_settings(app, s).await {
                    tracing::warn!(?e, "mock_enabled 리셋 실패");
                }
            }
            Ok(_) => {}
            Err(e) => tracing::warn!(?e, "부팅 시 settings 로드 실패"),
        }
    });
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn spawn_desktop_llm_loader(app: tauri::AppHandle) {
    use std::sync::Arc;

    use crate::llm::{mistralrs_backend::MistralRsSummarizer, LlmSummarizer};

    tauri::async_runtime::spawn(async move {
        match MistralRsSummarizer::load().await {
            Ok(s) => {
                let arc: Arc<dyn LlmSummarizer> = Arc::new(s);
                if app.state::<AppState>().summarizer.set(arc).is_err() {
                    tracing::warn!("요약 모델이 이미 등록되어 있음 — 중복 로드 무시");
                } else {
                    tracing::info!("Qwen3.6 요약 모델 활성화");
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "Qwen3.6 모델 로드 실패 — 요약은 비활성 상태");
            }
        }
    });
}
