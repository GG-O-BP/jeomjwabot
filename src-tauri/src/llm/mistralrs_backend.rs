use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use chrono::Utc;
use mistralrs::{GgufModelBuilder, Model, TextMessageRole, TextMessages};
use shared::{
    ChatEvent, DonationEvent, EventEnvelope, IpcError, LiveEvent, Platform, SubscriptionEvent,
    SummaryRequest, SummaryResponse,
};

use super::LlmSummarizer;

// 임시 fallback. Qwen3.6 GGUF는 `qwen35moe` 아키텍처라 mistral.rs v0.8.x가
// 로드 불가(loader 미작성). 동등한 활성 3B MoE인 Qwen3-30B-A3B로 우회.
// mistral.rs upstream에 Qwen3.5/3.6 MoE GGUF loader 들어오면 다시 3.6으로 전환.
const DEFAULT_FILENAME: &str = "Qwen3-30B-A3B-UD-Q4_K_XL.gguf";
const ENV_MODEL_DIR: &str = "JEOMJWABOT_MODEL_DIR";
const ENV_MODEL_FILE: &str = "JEOMJWABOT_MODEL_FILE";

pub struct MistralRsSummarizer {
    model: Arc<Model>,
}

impl MistralRsSummarizer {
    /// 환경변수 → 사용자 캐시 디렉터리 순으로 모델 위치를 결정한다.
    pub fn resolve_model_path() -> Result<(PathBuf, String), IpcError> {
        let dir = match std::env::var(ENV_MODEL_DIR) {
            Ok(s) if !s.is_empty() => PathBuf::from(s),
            _ => default_cache_dir()?,
        };
        let filename = std::env::var(ENV_MODEL_FILE)
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| DEFAULT_FILENAME.to_string());

        let full = dir.join(&filename);
        if !full.is_file() {
            return Err(IpcError::MissingConfig(format!(
                "Qwen3.6 GGUF 파일을 찾을 수 없습니다: {}",
                full.display()
            )));
        }
        Ok((dir, filename))
    }

    pub async fn load() -> Result<Self, IpcError> {
        let (dir, filename) = Self::resolve_model_path()?;
        let dir_str = dir
            .to_str()
            .ok_or_else(|| IpcError::Internal("모델 경로가 UTF-8이 아닙니다".into()))?
            .to_string();

        tracing::info!(
            model_dir = %dir_str,
            model_file = %filename,
            "Qwen3.6 GGUF 로드 시작 (CPU only)"
        );
        let started = Instant::now();
        let model = GgufModelBuilder::new(dir_str, vec![filename])
            .with_force_cpu()
            .with_logging()
            .build()
            .await
            .map_err(|e| IpcError::Internal(format!("LLM 로드 실패: {e}")))?;
        tracing::info!(
            elapsed_ms = started.elapsed().as_millis() as u64,
            "Qwen3.6 GGUF 로드 완료"
        );

        Ok(Self {
            model: Arc::new(model),
        })
    }
}

fn default_cache_dir() -> Result<PathBuf, IpcError> {
    dirs::cache_dir()
        .map(|p| p.join("jeomjwabot/models"))
        .ok_or_else(|| IpcError::Internal("사용자 캐시 디렉터리를 찾을 수 없습니다".into()))
}

#[async_trait]
impl LlmSummarizer for MistralRsSummarizer {
    async fn summarize(&self, req: SummaryRequest) -> Result<SummaryResponse, IpcError> {
        if req.events.is_empty() {
            return Ok(SummaryResponse {
                id: uuid::Uuid::new_v4().to_string(),
                text: "최근 활동 없음".into(),
                generated_at: Utc::now(),
            });
        }

        let max_chars = ((req.max_braille_cells / 2).max(8)) as usize;
        let system = format!(
            "당신은 라이브 방송의 채팅·후원·구독 이벤트를 시각장애인 점자단말기로 \
             읽힐 짧은 한국어 한 문장 요약으로 변환합니다. \
             규칙: (1) 한 문장. (2) 최대 {max_chars}자. (3) 존댓말. \
             (4) 이모지·영문 약어·한자 금지. (5) 금액은 한국어 단위 동반(예: \"천 원\"). \
             (6) 인사말 금지, 핵심 사실만. (7) 생각 과정·따옴표·괄호 출력 금지."
        );
        let user = render_events(&req.events);

        let msgs = TextMessages::new()
            .add_message(TextMessageRole::System, &system)
            .add_message(TextMessageRole::User, &user);

        let started = Instant::now();
        let resp = self
            .model
            .send_chat_request(msgs)
            .await
            .map_err(|e| IpcError::Internal(format!("LLM 추론 실패: {e}")))?;
        let raw = resp
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();
        let text = sanitize(&raw, max_chars)?;

        tracing::debug!(
            elapsed_ms = started.elapsed().as_millis() as u64,
            chars = text.chars().count(),
            "LLM 요약 생성"
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
    Ok(one_line.chars().take(max_chars).collect())
}
