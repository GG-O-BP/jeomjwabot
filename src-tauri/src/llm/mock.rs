use async_trait::async_trait;
use chrono::Utc;
use shared::{EventEnvelope, IpcError, LiveEvent, Platform, SummaryRequest, SummaryResponse};

use super::LlmSummarizer;

pub struct MockSummarizer;

#[async_trait]
impl LlmSummarizer for MockSummarizer {
    async fn summarize(&self, req: SummaryRequest) -> Result<SummaryResponse, IpcError> {
        let text = render(&req.events, req.max_braille_cells);
        Ok(SummaryResponse {
            id: uuid::Uuid::new_v4().to_string(),
            text,
            generated_at: Utc::now(),
        })
    }
}

fn render(events: &[EventEnvelope], max_cells: u32) -> String {
    if events.is_empty() {
        return "최근 활동 없음".into();
    }
    let max_chars = (max_cells / 2).max(8) as usize;

    let mut chat = 0u32;
    let mut donation = 0u32;
    let mut subscription = 0u32;
    let mut last_donor: Option<String> = None;
    let mut last_platform: Option<Platform> = None;

    for env in events {
        last_platform = Some(env.platform);
        match &env.payload {
            LiveEvent::Chat(_) => chat += 1,
            LiveEvent::Donation(d) => {
                donation += 1;
                if let Some(n) = &d.donator_nickname {
                    last_donor = Some(n.clone());
                }
            }
            LiveEvent::Subscription(_) => subscription += 1,
            LiveEvent::System(_) => {}
        }
    }

    let prefix = match last_platform {
        Some(Platform::Chzzk) => "치지직",
        Some(Platform::Cime) => "씨미",
        None => "",
    };

    let body = match (chat, donation, subscription) {
        (c, 0, 0) if c > 0 => format!("채팅 {c}건"),
        (c, d, 0) if d > 0 => match last_donor {
            Some(n) => format!("채팅 {c}건 후원 {d}건 ({n})"),
            None => format!("채팅 {c}건 후원 {d}건"),
        },
        (c, d, s) => format!("채팅 {c} 후원 {d} 구독 {s}"),
    };

    let head = if prefix.is_empty() {
        body
    } else {
        format!("{prefix} {body}")
    };
    head.chars().take(max_chars).collect()
}
