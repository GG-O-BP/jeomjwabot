use std::time::Duration;

use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use shared::{
    ChatEvent, CimeAuth, DonationEvent, DonationType, EventEnvelope, IpcError, LiveEvent, Platform,
    SubscriptionEvent, SystemEvent, SystemKind,
};
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::Message;
use url::Url;

use crate::auth;

const PING_INTERVAL_SECS: u64 = 60;

pub async fn run_cime(
    auth: CimeAuth,
    tx: broadcast::Sender<EventEnvelope>,
) -> Result<(), IpcError> {
    let session_url = auth::fetch_cime_session_url(&auth).await?;
    let session_key = extract_session_key(&session_url)?;

    tracing::info!(%session_key, "씨미 WS 연결 시도");
    let (ws, _) = tokio_tungstenite::connect_async(&session_url)
        .await
        .map_err(|e| IpcError::Network(format!("씨미 WS 연결 실패: {e}")))?;
    let (mut sink, mut stream) = ws.split();

    for event in ["chat", "donation", "subscription"] {
        if let Err(e) = auth::subscribe_cime_event(&auth, &session_key, event).await {
            tracing::warn!(?e, event, "씨미 이벤트 구독 실패");
        }
    }

    let _ = tx.send(envelope(LiveEvent::System(SystemEvent {
        kind: SystemKind::Connected,
        message: "씨미 연결됨".into(),
    })));

    let mut ping = tokio::time::interval(Duration::from_secs(PING_INTERVAL_SECS));
    ping.tick().await;

    loop {
        tokio::select! {
            _ = ping.tick() => {
                if let Err(e) = sink.send(Message::Text("{\"type\":\"PING\"}".into())).await {
                    tracing::warn!(?e, "씨미 PING 송신 실패");
                    break;
                }
            }
            msg = stream.next() => {
                match msg {
                    Some(Ok(Message::Text(t))) => {
                        if let Err(e) = dispatch(&t, &tx) {
                            tracing::warn!(?e, "씨미 메시지 디스패치 실패");
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(e)) => {
                        tracing::warn!(?e, "씨미 WS 오류");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    let _ = tx.send(envelope(LiveEvent::System(SystemEvent {
        kind: SystemKind::Disconnected,
        message: "씨미 연결 종료".into(),
    })));
    Ok(())
}

fn extract_session_key(raw: &str) -> Result<String, IpcError> {
    let parsed = Url::parse(raw).map_err(|e| IpcError::Protocol(format!("씨미 세션 URL: {e}")))?;
    parsed
        .query_pairs()
        .find_map(|(k, v)| (k == "sessionKey").then(|| v.into_owned()))
        .ok_or_else(|| IpcError::Protocol("씨미 세션 URL에 sessionKey 없음".into()))
}

fn envelope(payload: LiveEvent) -> EventEnvelope {
    EventEnvelope {
        id: uuid::Uuid::new_v4().to_string(),
        platform: Platform::Cime,
        received_at: Utc::now(),
        payload,
    }
}

fn dispatch(raw: &str, tx: &broadcast::Sender<EventEnvelope>) -> Result<(), IpcError> {
    let v: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| IpcError::Protocol(e.to_string()))?;

    if v.get("action").and_then(|x| x.as_str()) == Some("PONG") {
        return Ok(());
    }

    let Some(event) = v.get("event").and_then(|x| x.as_str()) else {
        return Ok(());
    };
    let data = v.get("data").cloned().unwrap_or(serde_json::Value::Null);
    let payload = match event {
        "CHAT" => LiveEvent::Chat(parse_chat(&data)?),
        "DONATION" => LiveEvent::Donation(parse_donation(&data)?),
        "SUBSCRIPTION" => LiveEvent::Subscription(parse_subscription(&data)?),
        _ => return Ok(()),
    };
    let _ = tx.send(envelope(payload));
    Ok(())
}

fn str_field(v: &serde_json::Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_string()
}

fn parse_chat(v: &serde_json::Value) -> Result<ChatEvent, IpcError> {
    let nickname = v
        .get("profile")
        .and_then(|p| p.get("nickname"))
        .and_then(|x| x.as_str())
        .unwrap_or("익명")
        .to_string();
    let content = str_field(v, "content");
    let raw_time = v.get("messageTime").and_then(|x| x.as_str());
    let message_time = raw_time
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);
    Ok(ChatEvent {
        nickname,
        content,
        user_role: None,
        verified: false,
        message_time,
    })
}

fn parse_donation(v: &serde_json::Value) -> Result<DonationEvent, IpcError> {
    let donator_nickname = v
        .get("donatorNickname")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());
    let amount = v
        .get("payAmount")
        .and_then(|x| {
            x.as_str()
                .and_then(|s| s.parse::<u64>().ok())
                .or(x.as_u64())
        })
        .unwrap_or(0);
    let message = str_field(v, "donationText");
    let donation_type = match v.get("donationType").and_then(|x| x.as_str()) {
        Some("VIDEO") => DonationType::Video,
        _ => DonationType::Chat,
    };
    Ok(DonationEvent {
        donator_nickname,
        amount,
        message,
        donation_type,
    })
}

fn parse_subscription(v: &serde_json::Value) -> Result<SubscriptionEvent, IpcError> {
    let subscriber_nickname = v
        .get("subscriberChannelName")
        .and_then(|x| x.as_str())
        .or_else(|| v.get("subscriberNickname").and_then(|x| x.as_str()))
        .unwrap_or("익명")
        .to_string();
    let tier_no = v.get("tierNo").and_then(|x| x.as_u64()).unwrap_or(1) as u8;
    let month = v.get("month").and_then(|x| x.as_u64()).unwrap_or(1) as u32;
    let message = v
        .get("subscriptionMessage")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());
    Ok(SubscriptionEvent {
        subscriber_nickname,
        tier_no,
        month,
        tier_name: None,
        message,
    })
}
