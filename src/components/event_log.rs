use leptos::prelude::*;
use shared::{DonationType, EventEnvelope, LiveEvent, UserRole};

// EventLog는 시각 사용자·전체 흐름 확인용 누적 영역이다.
// 점자 사용자는 SummaryPanel(role="status")의 짧은 요약만 받도록 설계 — EventLog는 한 줄
// 길이 제한을 두지 않고 원문 그대로 보존한다.
#[component]
pub fn EventLog(events: ReadSignal<Vec<EventEnvelope>>) -> impl IntoView {
    view! {
        <section aria-labelledby="event-log-heading">
            <h2 id="event-log-heading">"라이브 이벤트"</h2>
            <p role="status" aria-live="polite">
                {move || format!("총 {}건", events.with(|v| v.len()))}
            </p>
            <ol role="log" aria-live="polite" aria-label="실시간 라이브 이벤트">
                <For
                    each=move || events.get()
                    key=|env| env.id.clone()
                    children=move |env| view! { <li>{render_envelope(&env)}</li> }
                />
            </ol>
        </section>
    }
}

fn format_amount_ko(amount: u64) -> String {
    let raw = amount.to_string();
    let bytes = raw.as_bytes();
    let mut out = String::with_capacity(raw.len() + raw.len() / 3 + 1);
    for (idx, &b) in bytes.iter().enumerate() {
        if idx > 0 && (bytes.len() - idx).is_multiple_of(3) {
            out.push(',');
        }
        out.push(b as char);
    }
    out.push('원');
    out
}

fn render_envelope(env: &EventEnvelope) -> String {
    let plat = env.platform.label_ko();
    match &env.payload {
        LiveEvent::Chat(c) => {
            let role_prefix = match c.user_role {
                Some(UserRole::Streamer) => "스트리머 ",
                Some(UserRole::Manager) => "운영자 ",
                Some(UserRole::ChatManager) => "채팅 운영자 ",
                _ => "",
            };
            format!("{plat} {role_prefix}{} : {}", c.nickname, c.content)
        }
        LiveEvent::Donation(d) => {
            let nick = d.donator_nickname.as_deref().unwrap_or("익명");
            let kind = match d.donation_type {
                DonationType::Chat => "채팅 후원",
                DonationType::Video => "영상 후원",
            };
            let trailer = if d.message.is_empty() {
                String::new()
            } else {
                format!(" — {}", d.message)
            };
            format!(
                "{plat} {kind} {} : {}{}",
                nick,
                format_amount_ko(d.amount),
                trailer
            )
        }
        LiveEvent::Subscription(s) => {
            let tier = s
                .tier_name
                .as_deref()
                .map(|t| format!(" {t}"))
                .unwrap_or_default();
            let msg = s
                .message
                .as_deref()
                .filter(|m| !m.is_empty())
                .map(|m| format!(" — {m}"))
                .unwrap_or_default();
            format!(
                "{plat} {} {}개월 구독 (티어 {}{}){}",
                s.subscriber_nickname, s.month, s.tier_no, tier, msg
            )
        }
        LiveEvent::System(sys) => format!("{plat} 시스템 : {}", sys.message),
    }
}
