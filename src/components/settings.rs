use leptos::prelude::*;
use shared::{ChzzkSecrets, CimeSecrets, IpcError, Platform, SecretsPresence, Settings};

use crate::ipc;

#[derive(Default, Clone)]
struct SecretInputs {
    chzzk_client_secret: String,
    cime_access_token: String,
}

struct SavePayload {
    settings: Settings,
    secrets: SecretInputs,
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

    let save_msg = move || match save_action.value().get() {
        None if save_action.pending().get() => "저장 중...".to_string(),
        None => String::new(),
        Some(Ok(())) => "저장 및 연결 시도 완료".into(),
        Some(Err(e)) => format!("저장 실패: {e}"),
    };

    let chzzk_secret_hint = move || match presence.get() {
        None => "저장 상태 확인 중.",
        Some(p) if p.chzzk_present => "저장됨. 변경하려면 새 값 입력.",
        Some(_) => "아직 저장되지 않음.",
    };
    let cime_token_hint = move || match presence.get() {
        None => "저장 상태 확인 중.",
        Some(p) if p.cime_present => "저장됨. 변경하려면 새 값 입력.",
        Some(_) => "아직 저장되지 않음.",
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

                <fieldset>
                    <legend>"씨미 인증"</legend>
                    <div>
                        <label for="cime-token">"Access Token"</label>
                        <input id="cime-token" type="password" autocomplete="off"
                            aria-describedby="cime-token-hint"
                            prop:value=move || secret_inputs.with(|s| s.cime_access_token.clone())
                            on:input=move |ev| {
                                let v = event_target_value(&ev);
                                secret_inputs.update(|s| s.cime_access_token = v);
                            } />
                        <span id="cime-token-hint" class="hint">{move || cime_token_hint()}</span>
                    </div>
                </fieldset>

                <button type="submit">"저장 및 연결"</button>
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
    let cime_secret = (!secrets.cime_access_token.trim().is_empty()).then_some(CimeSecrets {
        access_token: secrets.cime_access_token,
    });

    if chzzk_secret.is_some() || cime_secret.is_some() {
        ipc::save_secrets(chzzk_secret, cime_secret).await?;
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
