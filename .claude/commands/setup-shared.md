---
description: 백엔드/프론트 공용 타입을 위한 shared 크레이트 부트스트랩 (최초 1회)
allowed-tools: Bash(ls:*), Bash(cargo:*), Bash(test:*), Read, Edit, Write
---

# shared 크레이트 부트스트랩

목적: 루트 CLAUDE.md 7번 원칙(`공용 타입은 shared 크레이트에 단일 정의`)을 만족시키기 위한 최초 셋업.

## 절차

1. `ls /home/ggobp/Workspace/jeomjwabot/shared 2>/dev/null` — 이미 존재하면 즉시 종료하고 사용자에게 "shared 크레이트는 이미 존재합니다 — 이 커맨드는 더 필요하지 않습니다"라고 안내.

2. **루트 `Cargo.toml` 수정**
   - 현재 `[workspace] members = ["src-tauri"]` → `members = ["src-tauri", "shared"]`로 확장.
   - 루트 `[dependencies]`에 `shared = { path = "shared" }` 추가.

3. **`src-tauri/Cargo.toml` 수정**
   - `[dependencies]`에 `shared = { path = "../shared" }` 추가.

4. **`shared/Cargo.toml` 작성**
   ```toml
   [package]
   name = "shared"
   version = "0.1.0"
   edition = "2021"

   [dependencies]
   serde = { version = "1", features = ["derive"] }
   thiserror = "1"
   ```

5. **`shared/src/lib.rs` 작성** — 골격 타입:
   ```rust
   //! 점자봇 백엔드(src-tauri)와 프론트(jeomjwabot-ui)가 공유하는 타입.
   //! 추가 시 양쪽 import 경로가 모두 컴파일되도록 유지하라.

   use serde::{Deserialize, Serialize};

   #[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
   pub struct ChannelId(pub String);

   #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
   pub enum Platform { Chzzk, Cime }

   #[derive(Debug, Clone, Serialize, Deserialize)]
   #[serde(tag = "kind", rename_all = "snake_case")]
   pub enum LiveEvent {
       Chat {
           platform: Platform,
           channel: ChannelId,
           sender: String,
           content: String,
           ts_ms: i64,
       },
       Donation {
           platform: Platform,
           channel: ChannelId,
           donor: Option<String>,
           amount_won: u64,
           message: String,
           ts_ms: i64,
       },
       Subscription {
           platform: Platform,
           channel: ChannelId,
           subscriber: String,
           tier: u8,
           months: u32,
           message: String,
           ts_ms: i64,
       },
   }

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct SummaryRequest {
       pub batch: Vec<LiveEvent>,
       /// 점자 1줄 폭(보통 32셀). 요약 길이 제약에 사용.
       pub max_braille_cells: u16,
       /// 사용자가 설정한 요약 주기 (초). 컨텍스트 정보로 LLM에 전달 가능.
       pub interval_seconds: u16,
   }

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct SummaryResponse {
       /// 한국어 평문. 점자 변환을 가정한 짧은 문장.
       pub text: String,
   }

   #[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
   #[serde(tag = "kind", content = "message")]
   pub enum AppError {
       #[error("auth: {0}")] Auth(String),
       #[error("network: {0}")] Network(String),
       #[error("llm: {0}")] Llm(String),
       #[error("braille: {0}")] Braille(String),
       #[error("api: {0}")] Api(String),
       #[error("other: {0}")] Other(String),
   }
   ```

6. `cargo check --workspace`로 컴파일 확인. 실패 시 메시지 그대로 노출하고 중단.

7. 끝에 한 줄 요약:
   ```
   shared crate created — exports: ChannelId, Platform, LiveEvent, SummaryRequest, SummaryResponse, AppError
   ```

8. 사용자에게 안내: "이제 컴포넌트와 명령에서 `use shared::{LiveEvent, SummaryRequest, …};`로 임포트하세요. 기존에 ad-hoc 정의된 동일 의미 타입이 있으면 `shared::*`로 교체하세요."
