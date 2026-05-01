use std::path::PathBuf;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use chrono::Utc;
use shared::{
    ChatEvent, DonationEvent, EventEnvelope, IpcError, LiveEvent, Platform, SubscriptionEvent,
    SummaryRequest, SummaryResponse,
};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::time::timeout;

use super::LlmSummarizer;

// Haiku 4.5: 한국어·짧은 요약·낮은 비용. 라이브 방송의 N초 cadence 요약에 적합.
// 모델 갈아끼우려면 JEOMJWABOT_CLAUDE_MODEL 환경변수.
const DEFAULT_MODEL: &str = "claude-haiku-4-5-20251001";

const ENV_BINARY: &str = "JEOMJWABOT_CLAUDE_BIN";
const ENV_MODEL: &str = "JEOMJWABOT_CLAUDE_MODEL";

// Haiku는 보통 1–3초 안에 응답. 60초는 네트워크 정체·서버 지연까지 봐주는 상한.
// 그 이상 지연되면 점자 cadence가 무너지므로 상위에서 다음 폴링으로 재시도.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

pub struct ClaudeCodeSummarizer {
    binary: PathBuf,
    model: String,
    /// 동시 호출은 점자 출력의 시간순 누적과 충돌하고, Anthropic 측 동시 요청 제한을 부담시킨다.
    /// 점자 cadence 자체가 직렬이라 직렬화로 충분.
    inference_lock: Mutex<()>,
}

impl ClaudeCodeSummarizer {
    fn resolve_binary() -> PathBuf {
        std::env::var(ENV_BINARY)
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("claude"))
    }

    pub async fn load() -> Result<Self, IpcError> {
        let binary = Self::resolve_binary();
        let model = std::env::var(ENV_MODEL)
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());

        // claude --version 으로 바이너리 존재 + 실행 가능 사전 검증.
        // Anthropic 인증 상태는 첫 -p 호출에서만 확인 가능 — 여기서 막을 수 없다.
        let output = Command::new(&binary)
            .arg("--version")
            .output()
            .await
            .map_err(|e| {
                IpcError::MissingConfig(format!(
                "claude 바이너리 실행 실패: {e}. PATH 확인 또는 {ENV_BINARY} 환경변수로 경로 지정."
            ))
            })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(IpcError::Internal(format!(
                "claude --version 실패: {stderr}"
            )));
        }

        tracing::info!(
            binary = %binary.display(),
            model = %model,
            version = %String::from_utf8_lossy(&output.stdout).trim(),
            "Claude Code headless 백엔드 활성화"
        );

        Ok(Self {
            binary,
            model,
            inference_lock: Mutex::new(()),
        })
    }
}

#[async_trait]
impl LlmSummarizer for ClaudeCodeSummarizer {
    async fn summarize(&self, req: SummaryRequest) -> Result<SummaryResponse, IpcError> {
        tracing::info!(
            events = req.events.len(),
            max_braille_cells = req.max_braille_cells,
            "요약 요청 수신"
        );

        if req.events.is_empty() {
            tracing::info!("이벤트 없음 — 추론 생략, 안내 문구 반환");
            return Ok(SummaryResponse {
                id: uuid::Uuid::new_v4().to_string(),
                text: "최근 활동 없음".into(),
                generated_at: Utc::now(),
            });
        }

        // 점자 단말기 32셀 1줄 = 한국어 약 16자. 한 문장 요약은 보통 30–50자라
        // 1줄에 강제로 우겨넣으면 단어 중간에서 잘린다 ("재미있대며 -" 사례).
        // 단말기는 멀티라인 스크롤이 자연스러우므로 2줄 분량(셀 × 2)을 LLM 목표로 준다.
        // 16셀 같은 작은 단말기는 floor 40자로 보장 — 의미 있는 한 문장의 최소 길이.
        let max_chars = (req.max_braille_cells * 2).max(40) as usize;
        // 1차 사용자는 화면을 못 보는 청자다. 옆자리 친구가 귓속말로 채팅창 분위기를
        // 알려주듯 *내용·반응·결*을 전달해야지, '채팅 N건·후원 N원' 같은 통계는
        // 방송을 즐기는 데 아무 도움이 안 된다. 모델이 카운트로 회피하는 걸 명시 차단.
        let system = format!(
            "당신은 라이브 방송 채팅창에서 일어나는 일을, 화면을 볼 수 없는 시청자가 \
             방송 흐름을 따라갈 수 있도록 한국어 한 문장으로 전달합니다.\n\
             \n\
             하지 말 것:\n\
             - '채팅 N건, 후원 N원, 구독 N건' 같은 통계·카운트 나열. \
             시각장애인이 방송을 즐기는 데 아무 도움이 안 된다.\n\
             - 사실만 무미건조하게 모은 것. 분위기·반응이 빠지면 의미 없다.\n\
             \n\
             해야 할 것:\n\
             - 채팅이 한 화제에 몰리면 그 화제와 시청자 반응을 묘사. \
             예: '신곡에 다들 환호', '방어카에 웃음 터짐', '재방송 요청 다수'.\n\
             - 후원·구독은 *닉네임과 메시지 내용*이 정보. 금액·개월수는 곁들이는 정도. \
             긴 메시지는 핵심만 한 어절로 압축. 예: '민규님 만 원, 신곡 칭찬'.\n\
             - 인상적인 반응 하나가 분위기를 대표하면 그걸 풀어 써도 좋음.\n\
             - 정말 별 일 없으면 솔직하게: '잔잔한 채팅', '조용한 분위기'.\n\
             \n\
             형식 규칙:\n\
             - 한 문장, 최대 {max_chars}자, 존댓말.\n\
             - 이모지·영문·한자·따옴표·괄호 금지 (점자 출력 잡음).\n\
             - 금액은 한국어 단위로 ('천 원', '만 원').\n\
             - 본문만 출력. 생각 과정·도구 사용 금지."
        );
        let user = render_events(&req.events);

        tracing::info!(
            system_chars = system.chars().count(),
            user_chars = user.chars().count(),
            max_output_chars = max_chars,
            "프롬프트 구성 완료"
        );

        let _guard = self.inference_lock.lock().await;
        tracing::info!("Claude Code 호출");
        let started = Instant::now();

        // claude headless: 도구 일절 차단(--allowed-tools "" + --max-turns 1) — 점좌봇은 텍스트만 필요.
        // --append-system-prompt 로 점자 출력 규칙 부착. --output-format text 로 plain stdout.
        let mut cmd = Command::new(&self.binary);
        cmd.arg("-p")
            .arg(&user)
            .arg("--model")
            .arg(&self.model)
            .arg("--output-format")
            .arg("text")
            .arg("--append-system-prompt")
            .arg(&system)
            .arg("--max-turns")
            .arg("1")
            .arg("--allowed-tools")
            .arg("");

        let output = match timeout(REQUEST_TIMEOUT, cmd.output()).await {
            Ok(Ok(o)) => o,
            Ok(Err(e)) => {
                return Err(IpcError::Internal(format!("claude 실행 실패: {e}")));
            }
            Err(_) => {
                return Err(IpcError::Internal(format!(
                    "Claude 응답이 {}초 안에 오지 않았습니다.",
                    REQUEST_TIMEOUT.as_secs()
                )));
            }
        };

        let infer_ms = started.elapsed().as_millis() as u64;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!(status = ?output.status, stderr = %stderr, "Claude 비정상 종료");
            return Err(IpcError::Internal(format!(
                "Claude 종료 코드 {}: {stderr}",
                output.status
            )));
        }

        let raw = String::from_utf8(output.stdout)
            .map_err(|e| IpcError::Internal(format!("Claude 응답 UTF-8 디코딩 실패: {e}")))?;
        tracing::info!(
            elapsed_ms = infer_ms,
            raw_chars = raw.chars().count(),
            "Claude 응답 수신"
        );
        drop(_guard);

        let text = match sanitize(&raw, max_chars) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!(raw = %raw, error = %e, "sanity check 실패 — 점자 출력 차단");
                return Err(e);
            }
        };
        tracing::info!(
            chars = text.chars().count(),
            "sanity check 통과 — 점자 출력 준비"
        );

        Ok(SummaryResponse {
            id: uuid::Uuid::new_v4().to_string(),
            text,
            generated_at: Utc::now(),
        })
    }
}

fn render_events(events: &[EventEnvelope]) -> String {
    let mut buf = String::with_capacity(events.len() * 64);
    buf.push_str("다음 이벤트들을 한 문장으로 요약하세요.\n");
    for env in events {
        let prefix = match env.platform {
            Platform::Chzzk => "치지직",
            Platform::Cime => "씨미",
        };
        match &env.payload {
            LiveEvent::Chat(ChatEvent {
                nickname, content, ..
            }) => {
                buf.push_str(&format!("[{prefix} 채팅] {nickname}: {content}\n"));
            }
            LiveEvent::Donation(DonationEvent {
                donator_nickname,
                amount,
                message,
                ..
            }) => {
                let name = donator_nickname.as_deref().unwrap_or("익명");
                buf.push_str(&format!("[{prefix} 후원] {name} {amount}원: {message}\n"));
            }
            LiveEvent::Subscription(SubscriptionEvent {
                subscriber_nickname,
                month,
                message,
                ..
            }) => {
                let msg = message.as_deref().unwrap_or("");
                buf.push_str(&format!(
                    "[{prefix} 구독] {subscriber_nickname} {month}개월차: {msg}\n"
                ));
            }
            LiveEvent::System(_) => {}
        }
    }
    buf
}

/// 점자 안전성 검증: 한 문장, 한국어 비율, 길이 컷.
/// 점자 단말기로 깨진 텍스트가 흘러가지 않게 막는 마지막 게이트.
fn sanitize(raw: &str, max_chars: usize) -> Result<String, IpcError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(IpcError::Internal("LLM 응답이 비어있습니다".into()));
    }
    let one_line = trimmed
        .split(['\n', '\r'])
        .next()
        .unwrap_or(trimmed)
        .trim()
        .to_string();
    let total = one_line.chars().count();
    if total == 0 {
        return Err(IpcError::Internal("LLM 응답이 비어있습니다".into()));
    }
    let hangul = one_line
        .chars()
        .filter(|c| ('가'..='힣').contains(c))
        .count();
    if (hangul as f32) / (total as f32) < 0.3 {
        return Err(IpcError::Internal(format!(
            "LLM 응답이 한국어가 아닙니다: {one_line}"
        )));
    }

    // 길이 정책:
    // - max_chars 는 LLM 에 전달한 *목표* 한도. LLM 이 약간 넘기는 건 자연스럽다.
    // - 1.5× 이내면 그대로 통과 (점자 단말기는 자동 줄바꿈).
    // - 그 이상이면 max_chars 근처의 단어·문장부호 경계에서 자른다.
    //   글자 중간에서 자르면 ("재미있대며 -") 점자 사용자에게 부서진 단어가 들린다.
    let ceiling = max_chars + max_chars / 2;
    if total <= ceiling {
        return Ok(one_line);
    }
    let head: String = one_line.chars().take(max_chars).collect();
    let cut_byte = head
        .rfind([' ', ',', '.', '!', '?', '\u{3002}', '\u{FF0C}'])
        .unwrap_or(head.len());
    Ok(head[..cut_byte].trim_end().to_string())
}
