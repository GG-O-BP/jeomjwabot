# Tauri 백엔드

이 디렉토리는 Rust 네이티브 측이다. 모바일(`#[cfg_attr(mobile, …)]`)과 데스크톱이 같은 코드 베이스에서 빌드된다.

## 명령어 추가 절차
1. `shared` 크레이트에 인자/응답 타입 추가 (있으면 재사용).
2. `src-tauri/src/`에 `#[tauri::command]` 함수 작성 — 가능하면 도메인 모듈로 분리 (`commands/chzzk.rs`, `commands/cime.rs`, `commands/llm.rs`).
3. `tauri::Builder::default().invoke_handler(tauri::generate_handler![...])`에 등록.
4. `capabilities/*.json`에 명령 허용 (필요 시).
5. `src/ipc.rs`에 동일 시그니처 타입 안전 래퍼 추가.

## 비동기 / 스레딩
- `async fn` 명령은 Tauri 자체 런타임에서 실행된다 — `tokio::spawn` 직접 호출 금지, `tauri::async_runtime::spawn` 사용.
- CPU 집약 작업: `tauri::async_runtime::spawn_blocking` 또는 `std::thread::spawn` + `tokio::sync::oneshot`/`mpsc`.
- 프론트로 결과 회신은 두 가지 방식:
  - 명령 반환값 (one-shot 응답).
  - `app.emit("event-name", payload)` (스트리밍 — 채팅 메시지 등).

## WebSocket 클라이언트
Chzzk와 Cime는 둘 다 표준 `wss://` (RFC 6455). **Socket.IO 라이브러리 절대 사용 금지** (Cime 문서가 명시).

추천:
- `tokio-tungstenite` (백엔드 표준)
- 책임 분리:
  - 세션 생성 + 토큰 갱신 (`auth.rs`)
  - WS 연결 + 자동 재연결 (`ws.rs`)
  - PING (Cime 1분 간격) — `tokio::time::interval`
  - 이벤트 디스패치 (`shared::LiveEvent`로 정규화 후 emit)

## 인증
- Chzzk: Client ID/Secret + (사용자 세션 시) Access Token.
- Cime: Client ID/Secret + Access Token (Bearer). OAuth 흐름은 `src-tauri/src/oauth/{cime.rs, loopback.rs}` + `src-tauri/src/commands/oauth.rs`.
- 토큰은 OS keyring에 저장 (`keyring` 크레이트, `secrets.rs`). **plain text 파일/JSON에 저장 금지**.
- Cime 토큰은 만료 60초 전이면 `commands/sources.rs::ensure_fresh_cime_token`이 자동 갱신.
- Refresh token 만료 처리 → 재로그인 UI emit (`oauth-progress` 이벤트).

## 디바이스 LLM 호출
- iOS: `Foundation Models`는 Swift. `swift-bridge` 또는 Tauri 플러그인의 `MobileBuilder` 훅으로 호출. (구현 예정)
- Android: AICore의 Gemini Nano는 Java/Kotlin. JNI 또는 `tauri-plugin-android` 패턴. (구현 예정)
- **Linux / Windows**: `mistralrs` v0.8 + Qwen3-30B-A3B GGUF (CPU 전용). 구현 `src-tauri/src/llm/mistralrs_backend.rs`. 의존성도 `[target.'cfg(any(target_os = "linux", target_os = "windows"))']`로 묶여 모바일 빌드 미포함.
- Rust 진입점: `trait LlmSummarizer` (`src-tauri/src/llm/mod.rs`). 모듈을 `#[cfg(target_os = ...)]`로 분기:
  - linux/windows → `mistralrs_backend`
  - ios → `ios` (예정)
  - android → `android` (예정)
  - 그 외 → `mock`
- 모델 로드는 `lib.rs::spawn_desktop_llm_loader`에서 비동기 1회. 결과는 `AppState.summarizer: OnceCell<Arc<dyn LlmSummarizer>>`.
- 출력 sanity: 한국어 비율 30% 미만·빈 응답·길이 초과는 reject (점자 단말기로 깨진 텍스트 송출 방지).

## 모바일 빌드 메모
- `tauri.conf.json`의 `bundle.identifier` 변경 시 iOS/Android 모두 재서명.
- `src-tauri/gen/` 폴더는 자동 생성 — 직접 편집 금지.
- iOS는 Apple Developer 계정과 provisioning profile 필요.
- Android는 `keystore` 별도 관리 (절대 커밋 금지 — `.gitignore`에 이미 있어야 한다).
- mobile cfg에서는 mistralrs 트리(~200 크레이트)를 끌어오지 않으므로 빌드 크기·시간 영향 없음 (cfg-target 분리로 보장).
