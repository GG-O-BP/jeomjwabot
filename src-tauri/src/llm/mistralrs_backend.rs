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

// CPU 추론 호환 모델. EXAONE 4.0 1.2B는 LG AI Research가 한국어 사전학습한
// on-device 전용 dense 모델로 점좌봇의 한국어 한 문장 요약에 적합하다.
// 주의: mistral.rs 0.8.1은 EXAONE 아키텍처를 명시 지원하지 않으므로
// `Unknown GGUF architecture 'exaone4'` panic 가능성. 미지원이 확인되면
// JEOMJWABOT_MODEL_FILE 환경변수로 Qwen3 dense (mistral.rs 명시 지원)로 폴백.
// candle 0.10 CPU 백엔드는 quantized MoE indexed_moe_forward 미구현이라
// Qwen3-30B-A3B / Qwen3.5-MoE / Qwen3.6-MoE / Gemma 4 26B-MoE 모두 panic.
const DEFAULT_FILENAME: &str = "EXAONE-4.0-1.2B-Q4_K_M.gguf";
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
                "EXAONE 4.0 GGUF 파일을 찾을 수 없습니다: {}",
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
            "EXAONE 4.0 GGUF 로드 시작 (CPU only)"
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
            "EXAONE 4.0 GGUF 로드 완료"
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

        let max_chars = ((req.max_braille_cells / 2).max(8)) as usize;
        let system = format!(
            "당신은 라이브 방송의 채팅·후원·구독 이벤트를 시각장애인 점자단말기로 \
             읽힐 짧은 한국어 한 문장 요약으로 변환합니다. \
             규칙: (1) 한 문장. (2) 최대 {max_chars}자. (3) 존댓말. \
             (4) 이모지·영문 약어·한자 금지. (5) 금액은 한국어 단위 동반(예: \"천 원\"). \
             (6) 인사말 금지, 핵심 사실만. (7) 생각 과정·따옴표·괄호 출력 금지."
        );
        let user = render_events(&req.events);
        tracing::info!(
            system_chars = system.chars().count(),
            user_chars = user.chars().count(),
            max_output_chars = max_chars,
            "프롬프트 구성 완료"
        );

        // Qwen3 chat template의 `enable_thinking=false`를 직접 전달.
        // <think>\n\n</think>\n\n 블록이 자동 삽입되어 reasoning 단계를 건너뛰고
        // 즉시 답변한다. 점자 한 문장 요약은 reasoning이 무용하므로 추론 시간을
        // 5–10배 단축한다.
        let msgs = TextMessages::new()
            .enable_thinking(false)
            .add_message(TextMessageRole::System, &system)
            .add_message(TextMessageRole::User, &user);

        tracing::info!("LLM 추론 시작 — send_chat_request 진입");
        let started = Instant::now();
        let resp = self.model.send_chat_request(msgs).await.map_err(|e| {
            tracing::error!(error = %e, "LLM 추론 실패");
            IpcError::Internal(format!("LLM 추론 실패: {e}"))
        })?;
        let infer_ms = started.elapsed().as_millis() as u64;
        let raw = resp
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();
        tracing::info!(
            elapsed_ms = infer_ms,
            raw_chars = raw.chars().count(),
            "LLM 추론 응답 수신"
        );

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
    Ok(one_line.chars().take(max_chars).collect())
}
