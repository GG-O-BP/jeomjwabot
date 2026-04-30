use leptos::prelude::*;
use shared::{
    ChzzkSecrets, CimeTokenStatus, IpcError, OAuthProgress, OAuthStage, Platform, SecretsPresence,
    Settings,
};

use crate::ipc;

#[derive(Default, Clone)]
struct SecretInputs {
    chzzk_client_secret: String,
    cime_client_secret: String,
}

struct SavePayload {
    settings: Settings,
    secrets: SecretInputs,
}

#[derive(Clone)]
struct CimeOauthInput {
    client_id: String,
    client_secret: String,
}

#[component]
pub fn SettingsForm(settings: RwSignal<Settings>) -> impl IntoView {
    let secret_inputs: RwSignal<SecretInputs> = RwSignal::new(SecretInputs::default());

    let presence = LocalResource::new(|| async {
        ipc::get_secrets_presence()
            .await
            .unwrap_or(SecretsPresence {
                chzzk_present: false,
                cime_present: false,
            })
    });

    let token_status = LocalResource::new(|| async {
        ipc::get_cime_token_status()
            .await
            .unwrap_or(CimeTokenStatus {
                access_token_present: false,
                client_secret_present: false,
                expires_at: None,
                scope: None,
            })
    });

    // OAuth 진행 상태 — aria-live region이 매 단계 한국어 메시지를 announce.
    let oauth_progress: RwSignal<Option<OAuthProgress>> = RwSignal::new(None);
    {
        let setter = oauth_progress;
        ipc::on_oauth_progress(move |p| setter.set(Some(p)));
    }

    let start_cime_source_action =
        Action::new_local(|_: &()| async { ipc::start_event_source(Platform::Cime).await });

    // 토큰 저장 완료 시 form의 비밀 입력을 비우고, 토큰 상태를 다시 가져오고,
    // 연결된 씨미 계정으로 곧장 이벤트 소스를 시작한다.
    Effect::new(move |_| {
        if let Some(p) = oauth_progress.get() {
            if matches!(p.stage, OAuthStage::Saved) {
                secret_inputs.update(|s| s.cime_client_secret = String::new());
                token_status.refetch();
                presence.refetch();
                start_cime_source_action.dispatch(());
            }
        }
    });

    let save_action = Action::new_local(|payload: &SavePayload| {
        let settings = payload.settings.clone();
        let secrets = payload.secrets.clone();
        async move { apply_and_save(settings, secrets).await }
    });

    Effect::new(move |_| {
        if save_action.value().get().is_some() {
            presence.refetch();
        }
    });

    let connect_cime_action = Action::new_local(|input: &CimeOauthInput| {
        let input = input.clone();
        async move { ipc::start_cime_oauth(input.client_id, input.client_secret).await }
    });

    let refresh_cime_action = Action::new_local(|_: &()| async {
        let r = ipc::refresh_cime_token().await;
        if r.is_ok() {
            // 갱신 직후 상태 갱신은 oauth-progress(Saved) Effect가 처리.
        }
        r
    });

    let cancel_cime_action = Action::new_local(|_: &()| async { ipc::cancel_cime_oauth().await });

    let save_msg = move || match save_action.value().get() {
        None if save_action.pending().get() => "설정을 저장하는 중입니다.".to_string(),
        None => String::new(),
        Some(Ok(())) => "설정 저장 및 연결 시도가 완료되었습니다.".into(),
        Some(Err(e)) => format!("저장 실패: {e}"),
    };

    let chzzk_secret_hint = move || match presence.get() {
        None => "저장 상태 확인 중.",
        Some(p) if p.chzzk_present => "저장됨. 변경하려면 새 값 입력.",
        Some(_) => "아직 저장되지 않음.",
    };

    // Memo<bool>: 4곳에서 동일 값을 보므로 eq-blocking이 의미가 있다.
    let oauth_in_flight = Memo::new(move |_| {
        matches!(
            oauth_progress.get().map(|p| p.stage),
            Some(OAuthStage::Starting)
                | Some(OAuthStage::AwaitingCallback)
                | Some(OAuthStage::Exchanging)
                | Some(OAuthStage::Saving)
        )
    });

    let oauth_message = move || oauth_progress.get().map(|p| p.message).unwrap_or_default();

    let cime_status_text = move || match token_status.get() {
        None => "씨미 토큰 상태를 불러오는 중입니다.".to_string(),
        Some(s) if !s.access_token_present => "씨미 계정이 아직 연결되지 않았습니다.".to_string(),
        Some(s) => {
            let expiry = s
                .expires_at
                .map(|t| format!("토큰 만료: {}.", t.format("%Y년 %m월 %d일 %H시 %M분")))
                .unwrap_or_else(|| "토큰 만료 시각 정보 없음.".into());
            let scope = s
                .scope
                .as_deref()
                .filter(|s| !s.is_empty())
                .map(|s| format!(" 권한: {s}."))
                .unwrap_or_default();
            format!("씨미 계정이 연결되어 있습니다. {expiry}{scope}")
        }
    };

    let refresh_available = move || {
        token_status
            .get()
            .map(|s| s.access_token_present && s.client_secret_present)
            .unwrap_or(false)
    };

    let cime_button_label = move || match token_status
        .get()
        .map(|s| s.access_token_present)
        .unwrap_or(false)
    {
        true => "씨미 계정 다시 연결",
        false => "씨미 계정 연결",
    };

    view! {
        <section aria-labelledby="settings-heading">
            <h2 id="settings-heading">"설정"</h2>
            <form on:submit=move |ev| {
                ev.prevent_default();
                save_action.dispatch(SavePayload {
                    settings: settings.get_untracked(),
                    secrets: secret_inputs.get_untracked(),
                });
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
                        "10초~600초. 너무 짧으면 점자 출력이 밀립니다."
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
                            prop:value=move || settings.with(|s| s.chzzk_client_id.clone().unwrap_or_default())
                            on:input=move |ev| {
                                let v = event_target_value(&ev);
                                settings.update(|s| s.chzzk_client_id = (!v.is_empty()).then_some(v));
                            } />
                    </div>
                    <div>
                        <label for="chzzk-secret">"Client Secret"</label>
                        <input id="chzzk-secret" type="password" autocomplete="off"
                            aria-describedby="chzzk-secret-hint"
                            prop:value=move || secret_inputs.with(|s| s.chzzk_client_secret.clone())
                            on:input=move |ev| {
                                let v = event_target_value(&ev);
                                secret_inputs.update(|s| s.chzzk_client_secret = v);
                            } />
                        <span id="chzzk-secret-hint" class="hint">{move || chzzk_secret_hint()}</span>
                    </div>
                </fieldset>

                <fieldset aria-describedby="cime-help">
                    <legend>"씨미 인증 (OAuth 자동화)"</legend>
                    <p id="cime-help" class="hint">
                        "씨미 개발자 포털에서 본인 앱의 Redirect URI로 "
                        <code>"http://127.0.0.1:8765/callback"</code>
                        "을 등록한 뒤 아래 단추를 눌러주세요. 시스템 브라우저에서 한 번만 승인하면 됩니다."
                    </p>
                    <div>
                        <label for="cime-cid">"Client ID"</label>
                        <input id="cime-cid" type="text" autocomplete="off"
                            prop:value=move || settings.with(|s| s.cime_client_id.clone().unwrap_or_default())
                            on:input=move |ev| {
                                let v = event_target_value(&ev);
                                settings.update(|s| s.cime_client_id = (!v.is_empty()).then_some(v));
                            } />
                    </div>
                    <div>
                        <label for="cime-secret">"Client Secret"</label>
                        <input id="cime-secret" type="password" autocomplete="off"
                            aria-describedby="cime-secret-hint"
                            prop:value=move || secret_inputs.with(|s| s.cime_client_secret.clone())
                            on:input=move |ev| {
                                let v = event_target_value(&ev);
                                secret_inputs.update(|s| s.cime_client_secret = v);
                            } />
                        <span id="cime-secret-hint" class="hint">
                            "연결 단추를 누르면 자격 증명 저장소에 보관됩니다."
                        </span>
                    </div>

                    <div role="group" aria-label="씨미 계정 동작">
                        <button
                            type="button"
                            on:click=move |_| {
                                let client_id = settings.with_untracked(|s| {
                                    s.cime_client_id.clone().unwrap_or_default()
                                });
                                let client_secret = secret_inputs
                                    .with_untracked(|s| s.cime_client_secret.clone());
                                connect_cime_action.dispatch(CimeOauthInput {
                                    client_id,
                                    client_secret,
                                });
                            }
                            prop:disabled=oauth_in_flight
                        >
                            {move || cime_button_label()}
                        </button>

                        <button
                            type="button"
                            on:click=move |_| { refresh_cime_action.dispatch(()); }
                            prop:disabled=move || !refresh_available() || oauth_in_flight.get()
                        >
                            "씨미 토큰 갱신"
                        </button>

                        <button
                            type="button"
                            on:click=move |_| { cancel_cime_action.dispatch(()); }
                            prop:disabled=move || !oauth_in_flight.get()
                        >
                            "씨미 인증 취소"
                        </button>
                    </div>

                    <p role="status" aria-live="polite" aria-atomic="true">
                        {move || cime_status_text()}
                    </p>
                    <p role="status" aria-live="polite" aria-atomic="true"
                       prop:aria-busy=oauth_in_flight>
                        {move || oauth_message()}
                    </p>
                </fieldset>

                <button type="submit">"설정 저장 및 치지직 연결"</button>
                <p role="status" aria-live="polite">{move || save_msg()}</p>
            </form>
        </section>
    }
}

async fn apply_and_save(settings: Settings, secrets: SecretInputs) -> Result<(), IpcError> {
    let chzzk_secret = (!secrets.chzzk_client_secret.trim().is_empty()).then_some(ChzzkSecrets {
        client_secret: secrets.chzzk_client_secret,
        access_token: None,
    });

    if chzzk_secret.is_some() {
        ipc::save_secrets(chzzk_secret, None).await?;
    }
    ipc::save_settings(settings.clone()).await?;

    if settings.mock_enabled {
        let _ = ipc::start_mock_source().await;
    } else {
        let _ = ipc::stop_mock_source().await;
    }

    let presence = ipc::get_secrets_presence()
        .await
        .unwrap_or(SecretsPresence {
            chzzk_present: false,
            cime_present: false,
        });

    let chzzk_ready = settings
        .chzzk_client_id
        .as_deref()
        .map(|s| !s.is_empty())
        .unwrap_or(false)
        && presence.chzzk_present;
    if chzzk_ready {
        let _ = ipc::start_event_source(Platform::Chzzk).await;
    } else {
        let _ = ipc::stop_event_source(Platform::Chzzk).await;
    }

    if presence.cime_present {
        let _ = ipc::start_event_source(Platform::Cime).await;
    } else {
        let _ = ipc::stop_event_source(Platform::Cime).await;
    }
    Ok(())
}
