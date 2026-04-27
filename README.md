# 점좌봇 (jeomjwabot)

> 시각장애인이 한소네 점자단말기로 라이브 방송을 따라잡을 수 있게 해주는 모바일 앱.

**점자 점역본 ⠨⠎⠢⠨⠣ ⠨⠎⠢⠱⠁⠘⠥⠒**: [README-braille.md](./README-braille.md) — 한글 점자(훈맹정음)로 점역한 README입니다 / ⠚⠣⠒⠈⠪⠂ ⠨⠎⠢⠨⠣(⠚⠍⠒⠑⠗⠶⠨⠎⠶⠪⠢)⠐⠥ ⠨⠎⠢⠱⠁⠚⠣⠒ README⠕⠃⠉⠕⠊⠣.

치지직(Chzzk) · 씨미(Cime)의 채팅 · 후원 · 구독 이벤트를 실시간으로 받아서, 사용자가 정한 N초 간격으로 **디바이스 안에서** LLM이 요약하고, 그 결과를 화면리더 / 점자 디스플레이로 흘려보낸다. 네트워크 너머로 방송 내용이 새어나가지 않도록 모든 요약은 단말기 위에서 수행된다.

---

## 1차 사용자

- 한소네 점자단말기 사용자 (시각장애인)
- 화면리더 + 점자 디스플레이 환경
- **시각 디자인은 부차적, 시멘틱 접근성이 1순위.**

라이브 채팅은 초당 수십 줄이 쏟아져서 점자 한 줄(약 16자)로는 따라갈 수 없다. 점좌봇은 그 흐름을 *사용자가 통제 가능한 박자*로 압축해주는 중간자다.

## 동작 흐름

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

1. 백엔드가 두 플랫폼의 WebSocket을 동시에 구독하고, 이벤트를 `shared::LiveEvent`로 정규화한다.
2. UI는 누적된 이벤트를 `aria-live` 영역에 노출하고, N초 타이머가 끝나면 디바이스 LLM에 요약을 요청한다.
3. 요약 결과는 점자 한 줄에 들어가는 길이로 잘려 화면리더 → 점자 디스플레이로 출력된다.

## 기술 스택

- **Tauri 2** — 모바일 타깃(iOS · Android), 데스크톱은 개발용
- **Leptos 0.8** — CSR(Wasm) 프론트엔드
- **wasm-bindgen 0.2 / serde 1 / serde-wasm-bindgen 0.6** — JS 경계 직렬화
- **Trunk + `cargo tauri`** — 빌드 파이프라인
- **On-device LLM** — iOS: Apple Foundation Models, Android: Gemini Nano (AICore)

## 워크스페이스 레이아웃

```
/Cargo.toml          # 루트 = jeomjwabot-ui (Leptos 프론트) + 워크스페이스 정의
/src/                # Leptos 프론트엔드 소스
/src-tauri/          # Tauri 백엔드 (워크스페이스 멤버)
/shared/             # 백/프론트 공용 타입 (첫 공용 타입 도입 시 생성)
/references/         # Chzzk / Cime API 공식 문서 — 작업 시 항상 우선 참조
```

`/setup-shared` 슬래시 커맨드로 `shared` 크레이트를 한 번에 부트스트랩할 수 있다.

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

dev 서버는 `localhost:1420`에 뜨며 `Trunk.toml`이 `src-tauri/`를 watch에서 제외한다.

### 사전 요구

- Rust stable (`rustup`), `wasm32-unknown-unknown` 타깃
- `trunk`, `cargo-tauri` (`cargo install trunk tauri-cli`)
- iOS 빌드: macOS + Xcode + Apple Developer 계정
- Android 빌드: Android Studio + NDK + 별도 keystore

## 접근성 원칙 (모든 UI 변경에 적용)

- 동적 영역에는 `role="log"`(시간순 누적) 또는 `role="status"`(1줄 상태) + `aria-live="polite"`. 권한 회수 같은 긴급 알림만 `assertive`.
- 입력 컨트롤은 `<label for=…>` 또는 `aria-labelledby`. placeholder만으로 라벨 대체 금지.
- 헤딩 계층(h1→h2→h3) 건너뛰지 않기.
- 자동 갱신 영역은 갱신 빈도 / 주기를 텍스트로 노출 (사용자가 예측 가능하게).
- 점자 라인(보통 32셀, 한국어 약 16자) 초과 텍스트는 의미 단위로 분할.
- 시각 변경(색 · 아이콘)이 시멘틱 변경 없이 유일한 정보 전달 수단이 되지 않게.

검증은 `/a11y-audit` 슬래시 커맨드.

## API 참조

이벤트 필드명 · 타입 · 예시는 추측 금지. 작업 전에 다음 문서를 1:1로 확인한다.

- `references/chzzk.md` — 치지직 세션 · 채팅 · 후원 · 구독
- `references/cime-sessions.html` — 씨미 세션 · WebSocket 연결 · 재연결 · PING
- `references/cime-chat.html` — 씨미 채팅 이벤트 본문
- `references/cime-donation.html` — 씨미 후원 이벤트 본문
- `references/cime-subscription.html` — 씨미 구독 이벤트 본문

> 두 플랫폼은 비슷하지만 다르다. 예: Chzzk `messageTime`은 ms `Int64`, Cime은 ISO 8601 문자열. Cime 구독 본문은 `subscriptionMessage`, Chzzk는 `month` + `tierName`. 한쪽만 보고 다른 쪽을 짐작하지 말 것.

## 개발 가이드

이 저장소에서 작업할 때 반드시 지켜야 하는 규칙은 [`CLAUDE.md`](./CLAUDE.md)에 정리되어 있다. 핵심:

- **Leptos 13대 원칙** — 컴포넌트는 1회 setup, 반응성은 signal/memo/effect, 비동기는 `Resource`, IPC는 타입 안전 래퍼 등.
- **Tauri IPC 규약** — 모든 invoke는 `src/ipc.rs` 한 곳에서 래핑, `shared` 크레이트의 동일 타입을 양쪽이 import.
- **점자 사용자 동선을 시각 사용자 기준으로 추측하지 말 것.** 의심되면 사용자에게 묻는다.

## 라이선스

[Mozilla Public License 2.0](./LICENSE) (MPL-2.0).
