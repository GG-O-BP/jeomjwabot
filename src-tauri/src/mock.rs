use std::time::Duration;

use chrono::Utc;
use rand::seq::SliceRandom;
use rand::Rng;
use shared::{
    ChatEvent, DonationEvent, DonationType, EventEnvelope, LiveEvent, Platform, SubscriptionEvent,
    SystemEvent, SystemKind,
};
use tokio::sync::broadcast;

const TICK_MS: u64 = 1000;

const NICKS: &[&str] = &[
    "김철수",
    "박영희",
    "이민준",
    "최서연",
    "정하은",
    "오지훈",
    "한가람",
];
const CHAT_LINES: &[&str] = &[
    "안녕하세요!",
    "오늘 방송 재미있네요",
    "ㅋㅋㅋㅋ",
    "스트리머님 화이팅",
    "다음 게임 뭐예요?",
    "점좌봇 잘 보여요",
];
const DONATION_LINES: &[&str] = &["응원합니다", "오늘도 즐방", "건강 챙기세요"];

pub async fn run_mock(tx: broadcast::Sender<EventEnvelope>) {
    let _ = tx.send(envelope(
        Platform::Chzzk,
        LiveEvent::System(SystemEvent {
            kind: SystemKind::Connected,
            message: "모의 데이터 켜짐".into(),
        }),
    ));

    let mut interval = tokio::time::interval(Duration::from_millis(TICK_MS));
    interval.tick().await;
    loop {
        interval.tick().await;
        let env = generate();
        if tx.send(env).is_err() {
            break;
        }
    }
}

fn envelope(platform: Platform, payload: LiveEvent) -> EventEnvelope {
    EventEnvelope {
        id: uuid::Uuid::new_v4().to_string(),
        platform,
        received_at: Utc::now(),
        payload,
    }
}

fn generate() -> EventEnvelope {
    let mut rng = rand::thread_rng();
    let platform = if rng.gen_bool(0.5) {
        Platform::Chzzk
    } else {
        Platform::Cime
    };
    let dice: f32 = rng.gen();
    let payload = if dice < 0.7 {
        LiveEvent::Chat(ChatEvent {
            nickname: NICKS
                .choose(&mut rng)
                .copied()
                .unwrap_or("익명")
                .to_string(),
            content: CHAT_LINES
                .choose(&mut rng)
                .copied()
                .unwrap_or("안녕하세요")
                .to_string(),
            user_role: None,
            verified: false,
            message_time: Utc::now(),
        })
    } else if dice < 0.9 {
        LiveEvent::Donation(DonationEvent {
            donator_nickname: Some(
                NICKS
                    .choose(&mut rng)
                    .copied()
                    .unwrap_or("익명")
                    .to_string(),
            ),
            amount: rng.gen_range(1_000..=50_000),
            message: DONATION_LINES
                .choose(&mut rng)
                .copied()
                .unwrap_or("응원")
                .to_string(),
            donation_type: DonationType::Chat,
        })
    } else {
        LiveEvent::Subscription(SubscriptionEvent {
            subscriber_nickname: NICKS
                .choose(&mut rng)
                .copied()
                .unwrap_or("익명")
                .to_string(),
            tier_no: 1,
            month: rng.gen_range(1..=12),
            tier_name: Some("티어1".into()),
            message: Some("앞으로도 응원합니다".into()),
        })
    };
    envelope(platform, payload)
}
