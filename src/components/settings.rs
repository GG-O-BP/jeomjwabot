use leptos::prelude::*;
use shared::{ChzzkAuth, CimeAuth, IpcError, Platform, Settings};

use crate::ipc;

#[component]
pub fn SettingsForm(settings: RwSignal<Settings>) -> impl IntoView {
    let save_action = Action::new_local(|payload: &Settings| {
        let payload = payload.clone();
        async move { apply_and_save(payload).await }
    });

    let save_msg = move || match save_action.value().get() {
        None if save_action.pending().get() => "저장 중...".to_string(),
        None => String::new(),
        Some(Ok(())) => "저장 및 연결 시도 완료".into(),
        Some(Err(e)) => format!("저장 실패: {e}"),
    };

    view! {
        <section aria-labelledby="settings-heading">
            <h2 id="settings-heading">"설정"</h2>
            <form on:submit=move |ev| {
                ev.prevent_default();
                save_action.dispatch(settings.get_untracked());
            }>
                <div>
                    <label for="channel-id">"채널 ID"</label>
                    <input id="channel-id" type="text" autocomplete="off"
                        prop:value=move || settings.with(|s| s.channel_id.clone())
                        on:input=move |ev| settings.update(|s| s.channel_id = event_target_value(&ev)) />
                </div>

                <div>
                    <label for="interval-secs">"요약 주기(초)"</label>
                    <input id="interval-secs" type="number" min="10" max="600"
                        aria-describedby="interval-help"
                        prop:value=move || settings.with(|s| s.summary_interval_secs).to_string()
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<u32>() {
                                settings.update(|s| s.summary_interval_secs = v.max(10));
                            }
                        } />
                    <span id="interval-help" class="hint">
                        "10초에서 600초 사이. 너무 짧으면 점자 단말기가 따라잡지 못합니다."
                    </span>
                </div>

                <div>
                    <label for="max-cells">"점자 셀 수 한도"</label>
                    <input id="max-cells" type="number" min="8" max="80"
                        aria-describedby="cells-help"
                        prop:value=move || settings.with(|s| s.max_braille_cells).to_string()
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<u32>() {
                                settings.update(|s| s.max_braille_cells = v.max(8));
                            }
                        } />
                    <span id="cells-help" class="hint">
                        "한소네는 보통 32셀, 한국어로 약 16자입니다."
                    </span>
                </div>

                <div>
                    <label for="mock-toggle">
                        <input id="mock-toggle" type="checkbox"
                            prop:checked=move || settings.with(|s| s.mock_enabled)
                            on:change=move |ev| settings.update(|s| s.mock_enabled = event_target_checked(&ev)) />
                        " 모의 데이터 사용 (토큰 없이 동선 검증)"
                    </label>
                </div>

                <fieldset>
                    <legend>"치지직 인증"</legend>
                    <div>
                        <label for="chzzk-cid">"Client ID"</label>
                        <input id="chzzk-cid" type="text" autocomplete="off"
                            prop:value=move || settings.with(|s| s.chzzk.as_ref().map(|c| c.client_id.clone()).unwrap_or_default())
                            on:input=move |ev| {
                                let v = event_target_value(&ev);
                                settings.update(|s| set_chzzk_field(s, |c| c.client_id = v));
                            } />
                    </div>
                    <div>
                        <label for="chzzk-secret">"Client Secret"</label>
                        <input id="chzzk-secret" type="password" autocomplete="off"
                            prop:value=move || settings.with(|s| s.chzzk.as_ref().map(|c| c.client_secret.clone()).unwrap_or_default())
                            on:input=move |ev| {
                                let v = event_target_value(&ev);
                                settings.update(|s| set_chzzk_field(s, |c| c.client_secret = v));
                            } />
                    </div>
                </fieldset>

                <fieldset>
                    <legend>"씨미 인증"</legend>
                    <div>
                        <label for="cime-token">"Access Token"</label>
                        <input id="cime-token" type="password" autocomplete="off"
                            prop:value=move || settings.with(|s| s.cime.as_ref().map(|c| c.access_token.clone()).unwrap_or_default())
                            on:input=move |ev| {
                                let v = event_target_value(&ev);
                                settings.update(|s| {
                                    s.cime = if v.is_empty() {
                                        None
                                    } else {
                                        Some(CimeAuth { access_token: v })
                                    };
                                });
                            } />
                    </div>
                </fieldset>

                <button type="submit">"저장 및 연결"</button>
                <p role="status" aria-live="polite">{save_msg}</p>
            </form>
        </section>
    }
}

fn set_chzzk_field(s: &mut Settings, mutate: impl FnOnce(&mut ChzzkAuth)) {
    let entry = s.chzzk.get_or_insert_with(|| ChzzkAuth {
        client_id: String::new(),
        client_secret: String::new(),
        access_token: None,
    });
    mutate(entry);
    if entry.client_id.is_empty() && entry.client_secret.is_empty() && entry.access_token.is_none()
    {
        s.chzzk = None;
    }
}

async fn apply_and_save(payload: Settings) -> Result<(), IpcError> {
    ipc::save_settings(payload.clone()).await?;

    if payload.mock_enabled {
        let _ = ipc::start_mock_source().await;
    } else {
        let _ = ipc::stop_mock_source().await;
    }

    if payload.chzzk.is_some() {
        let _ = ipc::start_event_source(Platform::Chzzk).await;
    } else {
        let _ = ipc::stop_event_source(Platform::Chzzk).await;
    }

    if payload.cime.is_some() {
        let _ = ipc::start_event_source(Platform::Cime).await;
    } else {
        let _ = ipc::stop_event_source(Platform::Cime).await;
    }
    Ok(())
}
