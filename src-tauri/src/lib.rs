mod auth;
mod commands;
mod llm;
mod mock;
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
        ])
        .run(tauri::generate_context!())
        .expect("점좌봇 실행 실패");
}
