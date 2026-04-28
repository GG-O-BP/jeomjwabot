# 점좌봇 (jeomjwabot)

시각장애인이 한소네 점자단말기로 라이브 방송을 따라잡을 수 있게 해주는 모바일 앱.
방송의 채팅·후원·구독 이벤트를 실시간 수신하고, 사용자가 정한 N초 간격으로 디바이스 LLM이 요약하고, 결과를 점자로 출력한다.

## 1차 사용자
- 한소네 점자단말기 사용자 (시각장애인)
- 화면리더 + 점자 디스플레이 환경
- **시각적 디자인은 부차적, 시멘틱 접근성이 1순위.**

## 아키텍처

```
[Chzzk / Cime API] ──WebSocket──▶ [Tauri Backend (Rust)]
                                          │
                                          ▼
                                 [shared 크레이트 (이벤트 타입)]
                                          │
                                          ▼
                          ┌───────────────┴───────────────┐
                          ▼                               ▼
                [Leptos UI (CSR/Wasm)]           [On-device LLM Bridge]
                          │                       iOS:     Apple Foundation Models
                          ▼                       Android: Gemini Nano (AICore)
                [한소네 점자 출력 어댑터]                   │
                                                          ▼
                                                  [요약 텍스트 → live region]
```

## 기술 스택
- Tauri 2 (mobile target: iOS, Android — 데스크톱은 개발용)
- Leptos 0.8 (CSR, `csr` feature)
- wasm-bindgen 0.2, serde 1, serde-wasm-bindgen 0.6
- 빌드: Trunk + `cargo tauri`
- 워크스페이스: 루트가 `jeomjwabot-ui` (frontend) + 멤버 `src-tauri` (+ 도입 예정 `shared`)

## 워크스페이스 레이아웃 (목표)

```
/Cargo.toml          # 루트 = jeomjwabot-ui 패키지 + 워크스페이스 정의
/src/                # Leptos 프론트엔드 소스
/src-tauri/          # Tauri 백엔드 (워크스페이스 멤버)
/shared/             # 백/프론트 공용 타입 (현재 없음 — 첫 공용 타입 도입 시 생성)
/references/         # Chzzk/Cime API 공식 문서 — API 작업 시 항상 우선 참조
```

> 첫 공용 타입(예: `LiveEvent`, `SummaryRequest`)이 등장하는 순간 `shared` 크레이트를 만들고, ui와 src-tauri 양쪽이 path dependency로 import한다. `/setup-shared` 커맨드로 한 번에 부트스트랩.

## 빌드 & 실행

| 동작 | 명령 |
|---|---|
| 데스크톱 dev (HMR) | `cargo tauri dev` |
| 프론트만 dev | `trunk serve` |
| 프론트 빌드 | `trunk build` |
| 데스크톱 릴리즈 | `cargo tauri build` |
| iOS dev | `cargo tauri ios dev` |
| Android dev | `cargo tauri android dev` |
| 타입 체크 | `cargo check --workspace --all-targets` |
| 린트 | `cargo clippy --workspace --all-targets -- -D warnings` |
| 포맷 | `cargo fmt --all` |

## API 참조 — 코드 작성 전 필수

**이벤트 수신 (WebSocket)**
- `references/chzzk.md` — 치지직 세션·채팅·후원·구독 (소켓 메시지 본문 스키마)
- `references/cime-sessions.html` — 씨미 세션·WebSocket 연결·재연결·PING
- `references/cime-chat.html` — 씨미 채팅 이벤트 본문
- `references/cime-donation.html` — 씨미 후원 이벤트 본문
- `references/cime-subscription.html` — 씨미 구독 이벤트 본문

**채팅 송신 (REST) + OAuth 인증**
- `references/chzzk-authorization.md` — 치지직 OAuth (인증 코드 발급, Access/Refresh Token, scope)
- `references/chzzk-input.md` — 치지직 채팅 메시지 전송·공지·설정·메시지 숨기기
- `references/cime-authentication.html` — 씨미 인증 (Client ID/Secret + Access Token Bearer 두 방식)
- `references/cime-input.html` — 씨미 채팅 메시지 전송·설정·`senderType`(APP/USER)

이벤트 필드명·타입·예시는 추측 금지. 반드시 위 문서 스키마와 1:1로 일치시킨다. **두 플랫폼은 비슷하지만 다르다** — 예: Chzzk `messageTime`은 ms `Int64`, Cime은 ISO 8601 문자열. Cime 구독 본문은 `subscriptionMessage`, Chzzk는 `month` + `tierName`. Chzzk 송신 엔드포인트는 `POST /open/v1/chats/send`, Cime은 `POST /api/openapi/open/v1/chats/send` — prefix가 다르다. 한쪽만 보고 다른 쪽을 짐작하지 말 것.

## 채팅 송신 정책 — 항상 사용자 본인(USER) 명의

점좌봇이 채팅을 보낼 때는 **양 플랫폼 모두 사용자 본인 계정 명의로만** 전송한다.
- **치지직**: 사용자 OAuth 인증 후 발급된 Access Token + scope `채팅 메시지 쓰기` 사용. 다른 옵션 없음.
- **씨미**: `POST /api/openapi/open/v1/chats/send` body의 `senderType`을 **반드시 `"USER"` 명시**. 기본값 `APP`(애플리케이션 소유자 명의 봇 송신)은 **금지**. 점좌봇은 봇이 아니라 사용자의 어시스턴스 도구이므로 봇 명의 송신은 사용자 의도와 어긋난다.
- 두 플랫폼 차이를 흡수하는 `shared` 송신 타입은 발신자 정체성을 항상 USER 고정으로 설계.

## Leptos 13대 원칙 (엄격 준수)

협상 대상이 아니다. 위배가 발견되면 즉시 고친다. 자동 검증은 `/leptos-audit`.

1. **컴포넌트는 1회 실행 setup 함수**다. 본문은 매 리렌더가 아니라 마운트 시점에 1번 호출된다. 본문에서 매번 fetch/IPC, 매번 새 closure를 props로 만드는 패턴 금지.
2. **반응성은 signal/memo/effect로만 발생한다.** 일반 변수·`Mutex`·`RefCell`로 UI를 갱신하지 않는다.
3. **`view!` 안의 reactive 값은 `move || signal.get()` 클로저로 감싼다.** 그냥 `{ signal.get() }`은 1회 평가 — 반응성이 사라진다.
4. **파생 값은 기본 derived signal**(클로저), **비용이 크거나 동등성 차단(eq blocking)이 필요할 때만 `Memo`**. 모든 파생을 `Memo`로 감싸지 않는다.
5. **`Effect`는 외부 세계 동기화 전용**이다. localStorage 쓰기, IPC 호출, 점자 단말기 출력 등. **signal에서 signal을 파생할 때 `Effect`를 쓰지 마라** — 그건 derived signal 또는 `Memo`의 일.
6. **비동기는 `Resource`로 reactive graph에 통합한다.** `spawn_local`을 직접 부르는 코드는 새로 추가하지 않는다 (기존 보일러플레이트는 마이그레이션 대상).
7. **공용 타입은 `shared` 크레이트에 단일 정의**한다. `LiveEvent`, `SummaryRequest` 같은 타입을 ui와 src-tauri가 각자 정의하지 않는다.
8. **`ReadSignal`/`WriteSignal`/`RwSignal`은 의도에 따라 구분**한다. 자식이 읽기만 하면 `ReadSignal`만 받는다. `RwSignal`은 진짜 양방향이 필요할 때만.
9. **동적 리스트는 `<For>` + 안정된 `key`**. `iter().map().collect_view()`로 동적 리스트를 만들지 않는다 (key 없으면 화면리더가 항목 변화를 잘못 announce한다).
10. **점자/스크린리더 사용자가 1차 사용자**다. 의미 있는 ARIA 속성과 `aria-live` region을 우선 설계한다. 시각 스타일은 그 다음.
11. **CPU 집약 작업(요약 토큰화, 히스토리 압축 등)은 별도 스레드**. 결과는 `Send` 가능한 signal로 회신한다. wasm은 web worker, 데스크톱/모바일은 `tauri::async_runtime::spawn_blocking`.
12. **Tauri IPC는 타입 안전 래퍼로만 호출한다.** `invoke("greet", …)` 같은 stringly-typed 호출은 `src/ipc.rs`의 한 함수 안에 격리한다. 컴포넌트는 `ipc::greet(name).await?`만 본다.
13. **`Suspense` fallback은 점자로 의미 있는 텍스트**여야 한다. "Loading..." 대신 "치지직 채팅 연결 중", "요약 생성 중 — 5초 후 다시 출력" 같이 사용자 행동을 가능하게 하는 메시지.

## 접근성 체크리스트 (모든 UI 변경에 적용)
- 새 동적 영역에는 `role="log"` (시간순 누적) 또는 `role="status"` (1줄 상태) + `aria-live="polite"`. 권한 회수 같은 긴급 알림만 `assertive`.
- 입력 컨트롤은 `<label for=…>` 또는 `aria-labelledby`. placeholder만으로 라벨 대체 금지.
- 헤딩 계층(h1→h2→h3) 건너뛰지 않기.
- 자동 갱신 영역은 갱신 빈도/주기를 텍스트로 노출 (사용자가 예측 가능하게).
- 점자 라인(보통 32셀, 한국어 약 16자) 초과 텍스트는 의미 단위로 분할.
- 시각 변경(색·아이콘)이 시멘틱 변경 없이 유일한 정보 전달 수단이 되지 않게.

자동 검증은 `/a11y-audit`.

## Tauri IPC 규약
- `src/ipc.rs` (단일 모듈)에 모든 invoke 래퍼를 모은다.
- 각 래퍼는 `pub async fn name(args: Args) -> Result<Output, IpcError>` 형태.
- 인자/응답 타입은 `shared` 크레이트의 동일 타입을 양쪽이 import.
- `src-tauri` 쪽 `#[tauri::command]` 함수와 IPC 래퍼가 1:1 대응. 한쪽이 변하면 다른 쪽도 함께.
- 검증은 `tauri-ipc-reviewer` 에이전트.

## 디바이스 LLM 브릿지
- **iOS**: Apple Foundation Models (Swift API). `swift-bridge` 또는 Tauri 모바일 플러그인의 `MobileBuilder` 훅.
- **Android**: Gemini Nano via AICore (Java/Kotlin). JNI 또는 `tauri-plugin-android` 패턴.
- Rust 측 진입점은 `trait LlmSummarizer { async fn summarize(&self, req: SummaryRequest) -> Result<SummaryResponse, AppError>; }`.
- iOS/Android 구현체를 `#[cfg(target_os = "ios")]` / `#[cfg(target_os = "android")]`로 분기. 데스크톱 dev에는 mock 구현.
- 두 플랫폼의 차이는 `shared::SummaryRequest` / `shared::SummaryResponse` 단일 타입으로 흡수.
- 출력 언어는 한국어 고정. 길이는 항상 점자 폭(`max_braille_cells`)을 의식한 짧은 문장.

## 한소네 점자 출력
- 기본 가정: 한소네는 USB/블루투스로 OS 화면리더와 연동된다 (별도 SDK 호출 X). 따라서 1차 출력 채널은 **OS 화면리더가 읽도록 의미 있는 DOM**을 만드는 것.
- 직접 점자 통신이 필요한 시나리오(예: 사용자 화면을 끄고 점자에만 정적 페이지 송출)는 별도 모듈로 분리. 미정 시점까지는 화면리더 경유로만.

## 코딩 규약
- 새 파일을 만들기보다 기존 파일에 추가한다.
- 주석은 *왜*가 비자명할 때만. *무엇을*은 식별자 이름이 말한다.
- `unwrap()`은 wasm 진입점·빌드 시점 검증된 상수·테스트에서만. 그 외 fallible은 `?`.
- `#[derive(...)]`에 `Clone`, `PartialEq`를 무조건 붙이지 않는다 — 필요할 때만.
- 한국어 식별자/주석 OK. 단 공개 API 식별자는 영문.

## 슬래시 커맨드
- `/check` — `cargo fmt --check`, `cargo check`, `cargo clippy`를 한 번에.
- `/leptos-audit` — 변경된 `.rs`를 13대 원칙에 비추어 감사.
- `/a11y-audit` — 변경된 `view!` 매크로의 ARIA·점자 적합성 감사.
- `/setup-shared` — `shared` 크레이트 부트스트랩 (최초 공용 타입 도입 시).

## 절대 하지 말 것
- 빈 컴포넌트를 만들고 `TODO`만 남긴 채 다른 작업으로 이동.
- 보일러플레이트 `greet` 명령을 그대로 둔 채 새 IPC 명령을 추가 (먼저 정리 또는 완전히 제거).
- 점자 사용자 동선을 시각 사용자 기준으로 추측. 의심되면 사용자에게 묻는다.
- 디바이스 LLM의 출력을 raw 그대로 렌더 — 한 글자 깨짐도 점자에서는 단어 전체가 무너진다. 항상 정상 한국어인지 sanity check.
- API 필드명·타입을 references/ 문서 확인 없이 추측.
- 씨미 채팅 송신에서 `senderType`을 생략하거나 `"APP"`으로 보내기. 점좌봇은 항상 `"USER"` 명시.
- OAuth Access Token / Refresh Token을 일반 설정 파일·localStorage에 저장. 토큰류는 `tauri-plugin-stronghold`(또는 OS keyring) 경유로만 보관 (PR #1 인프라 재사용).
