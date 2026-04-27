use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use shared::{
    ChatEvent, ChzzkAuth, DonationEvent, DonationType, EventEnvelope, IpcError, LiveEvent,
    Platform, SubscriptionEvent, SystemEvent, SystemKind, UserRole,
};
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::Message;
use url::Url;

use crate::auth;
use crate::ws::engineio::{self, EnginePacket, SocketIoPacket};

pub async fn run_chzzk(
    auth: ChzzkAuth,
    tx: broadcast::Sender<EventEnvelope>,
) -> Result<(), IpcError> {
    let session_url = auth::fetch_chzzk_session_url(&auth).await?;
    let ws_url = build_ws_url(&session_url)?;
    tracing::info!(%ws_url, "치지직 WS 연결 시도");

    let (ws, _) = tokio_tungstenite::connect_async(ws_url.as_str())
        .await
        .map_err(|e| IpcError::Network(format!("치지직 WS 연결 실패: {e}")))?;
    let (mut sink, mut stream) = ws.split();

    while let Some(msg) = stream.next().await {
        let raw = match msg {
            Ok(Message::Text(t)) => t,
            Ok(Message::Close(_)) => break,
            Ok(_) => continue,
            Err(e) => {
                tracing::warn!(?e, "치지직 WS 수신 오류");
                break;
            }
        };

        match engineio::parse(&raw) {
            Ok(EnginePacket::Open) => {
                if let Err(e) = sink.send(Message::Text("40".into())).await {
                    tracing::warn!(?e, "치지직 Connect(40) 송신 실패");
                    break;
                }
            }
            Ok(EnginePacket::Ping(payload)) => {
                let pong = format!("3{payload}");
                if let Err(e) = sink.send(Message::Text(pong)).await {
                    tracing::warn!(?e, "치지직 Pong(3) 송신 실패");
                    break;
                }
            }
            Ok(EnginePacket::Message(SocketIoPacket::Event(payload))) => {
                if let Err(e) = handle_event(payload, &auth, &tx).await {
                    tracing::warn!(?e, "치지직 이벤트 처리 실패");
                }
            }
            Ok(EnginePacket::Close) => break,
            Ok(_) => {}
            Err(e) => tracing::warn!(?e, "치지직 packet 파싱 실패"),
        }
    }

    let _ = tx.send(envelope(LiveEvent::System(SystemEvent {
        kind: SystemKind::Disconnected,
        message: "치지직 연결 종료".into(),
    })));
    Ok(())
}

fn build_ws_url(raw: &str) -> Result<Url, IpcError> {
    let mut url =
        Url::parse(raw).map_err(|e| IpcError::Protocol(format!("치지직 세션 URL: {e}")))?;
    let _ = url.set_scheme(if url.scheme() == "http" { "ws" } else { "wss" });
    url.set_path("/socket.io/");
    {
        let mut q = url.query_pairs_mut();
        q.append_pair("EIO", "3");
        q.append_pair("transport", "websocket");
    }
    Ok(url)
}

fn envelope(payload: LiveEvent) -> EventEnvelope {
    EventEnvelope {
        id: uuid::Uuid::new_v4().to_string(),
        platform: Platform::Chzzk,
        received_at: Utc::now(),
        payload,
    }
}

async fn handle_event(
    payload: &str,
    auth: &ChzzkAuth,
    tx: &broadcast::Sender<EventEnvelope>,
) -> Result<(), IpcError> {
    let arr: serde_json::Value =
        serde_json::from_str(payload).map_err(|e| IpcError::Protocol(e.to_string()))?;
    let arr = arr
        .as_array()
        .ok_or_else(|| IpcError::Protocol("Socket.IO event payload는 배열".into()))?;
    let name = arr
        .first()
        .and_then(|x| x.as_str())
        .ok_or_else(|| IpcError::Protocol("이벤트 이름 없음".into()))?;
    let data = arr.get(1).cloned().unwrap_or(serde_json::Value::Null);

    match name {
        "SYSTEM" => handle_system(&data, auth, tx).await,
        "CHAT" => {
            let _ = tx.send(envelope(LiveEvent::Chat(parse_chat(&data))));
            Ok(())
        }
        "DONATION" => {
            let _ = tx.send(envelope(LiveEvent::Donation(parse_donation(&data))));
            Ok(())
        }
        "SUBSCRIPTION" => {
            let _ = tx.send(envelope(LiveEvent::Subscription(parse_subscription(&data))));
            Ok(())
        }
        _ => Ok(()),
    }
}

async fn handle_system(
    data: &serde_json::Value,
    auth: &ChzzkAuth,
    tx: &broadcast::Sender<EventEnvelope>,
) -> Result<(), IpcError> {
    let kind = data.get("type").and_then(|x| x.as_str()).unwrap_or("");
    let inner = data.get("data");
    match kind {
        "connected" => {
            let session_key = inner
                .and_then(|x| x.get("sessionKey"))
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string();
            let _ = tx.send(envelope(LiveEvent::System(SystemEvent {
                kind: SystemKind::Connected,
                message: "치지직 연결됨".into(),
            })));
            for ev in ["chat", "donation", "subscription"] {
                if let Err(e) = auth::subscribe_chzzk_event(auth, &session_key, ev).await {
                    tracing::warn!(?e, ev, "치지직 이벤트 구독 실패");
                }
            }
        }
        "subscribed" => {
            let _ = tx.send(envelope(LiveEvent::System(SystemEvent {
                kind: SystemKind::Subscribed,
                message: "치지직 구독 시작".into(),
            })));
        }
        "unsubscribed" => {
            let _ = tx.send(envelope(LiveEvent::System(SystemEvent {
                kind: SystemKind::Unsubscribed,
                message: "치지직 구독 해제".into(),
            })));
        }
        "revoked" => {
            let _ = tx.send(envelope(LiveEvent::System(SystemEvent {
                kind: SystemKind::Revoked,
                message: "치지직 권한이 회수되었습니다 — 다시 로그인 필요".into(),
            })));
        }
        _ => {}
    }
    Ok(())
}

fn parse_chat(v: &serde_json::Value) -> ChatEvent {
    let nickname = v
        .get("profile")
        .and_then(|p| p.get("nickname"))
        .and_then(|x| x.as_str())
        .unwrap_or("익명")
        .to_string();
    let content = v
        .get("content")
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_string();
    let user_role = v
        .get("userRoleCode")
        .and_then(|x| x.as_str())
        .map(|s| match s {
            "streamer" => UserRole::Streamer,
            "streaming_channel_manager" => UserRole::Manager,
            "streaming_chat_manager" => UserRole::ChatManager,
            _ => UserRole::Common,
        });
    let verified = v
        .get("profile")
        .and_then(|p| p.get("verifiedMark"))
        .and_then(|x| x.as_bool())
        .unwrap_or(false);
    let message_time = v
        .get("messageTime")
        .and_then(|x| x.as_i64())
        .and_then(DateTime::<Utc>::from_timestamp_millis)
        .unwrap_or_else(Utc::now);
    ChatEvent {
        nickname,
        content,
        user_role,
        verified,
        message_time,
    }
}

fn parse_donation(v: &serde_json::Value) -> DonationEvent {
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
    let message = v
        .get("donationText")
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_string();
    let donation_type = match v.get("donationType").and_then(|x| x.as_str()) {
        Some("VIDEO") => DonationType::Video,
        _ => DonationType::Chat,
    };
    DonationEvent {
        donator_nickname,
        amount,
        message,
        donation_type,
    }
}

fn parse_subscription(v: &serde_json::Value) -> SubscriptionEvent {
    let subscriber_nickname = v
        .get("subscriberNickname")
        .and_then(|x| x.as_str())
        .unwrap_or("익명")
        .to_string();
    let tier_no = v.get("tierNo").and_then(|x| x.as_u64()).unwrap_or(1) as u8;
    let month = v.get("month").and_then(|x| x.as_u64()).unwrap_or(1) as u32;
    let tier_name = v
        .get("tierName")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());
    SubscriptionEvent {
        subscriber_nickname,
        tier_no,
        month,
        tier_name,
        message: None,
    }
}
