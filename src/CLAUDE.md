# Leptos 프론트엔드 (CSR)

이 디렉토리는 Tauri webview에 로드되는 Leptos 0.8 CSR 코드다. 루트 `CLAUDE.md`의 13대 원칙을 여기서 가장 엄격히 적용한다.

## 빠른 자가 점검 (커밋 전)

- [ ] 모든 컴포넌트 함수는 인자 외 closure capture로만 외부 상태에 접근.
- [ ] `view!` 안의 signal은 `move || s.get()` 또는 callable form `move || s()`로 감쌌다.
- [ ] derived signal로 충분한 곳에 `Memo`를 쓰지 않았다.
- [ ] `Effect::new`로 signal-to-signal 파생을 만들지 않았다 (외부 세계 동기화에만).
- [ ] 비동기 fetch/IPC는 `Resource::new`. raw `spawn_local`을 새로 추가하지 않았다.
- [ ] 동적 리스트는 `<For each=… key=… children=… />`. key는 안정 ID (배열 인덱스 X).
- [ ] 동적 영역에 `aria-live` / `role` 시멘틱이 들어 있다.
- [ ] `Suspense` fallback이 의미 있는 한국어.

## IPC 호출 규약
직접 `invoke(...)` 호출은 **`src/ipc.rs` 안에서만** 허용된다. 컴포넌트는 `crate::ipc::summarize_recent(...)` 같은 타입 안전 함수만 본다.

새 IPC 명령 추가 절차:
1. `shared` 크레이트에 인자/응답 타입 추가.
2. `src-tauri/src/`에 `#[tauri::command]` 함수 작성.
3. `tauri::Builder` invoke_handler에 등록.
4. `src/ipc.rs`에 동일 시그니처 래퍼 추가.
5. 컴포넌트는 래퍼만 호출.

## 점자 라이브 영역 패턴

```rust
// 채팅 누적 — 시간순
view! {
    <section role="log" aria-live="polite" aria-label="실시간 채팅 요약">
        <Suspense fallback=move || view! { <p>"채팅 연결 중"</p> }>
            <For
                each=move || summaries.get()
                key=|s| s.id.clone()
                children=move |s| view! { <p>{s.text}</p> }
            />
        </Suspense>
    </section>
}

// 1줄 상태 — 가장 최근 값만
view! {
    <p role="status" aria-live="polite" aria-label="요약 주기">
        {move || format!("{}초마다 요약", interval.get())}
    </p>
}
```

## 흔한 실수와 정답

| ❌ 안티패턴 | ✅ 정답 |
|---|---|
| `view! { <p>{count.get()}</p> }` | `view! { <p>{move \|\| count.get()}</p> }` |
| `let doubled = Memo::new(\|_\| count.get() * 2);` (단순 산술) | `let doubled = move \|\| count.get() * 2;` |
| `Effect::new(move \|_\| set_b.set(a.get() + 1));` | `let b = move \|\| a.get() + 1;` (derived) |
| `spawn_local(async move { ... })` | `Resource::new(move \|\| input.get(), \|i\| async move { ... })` |
| `RwSignal<T>`을 read-only 자식에 prop | `ReadSignal<T>` |
| `items.iter().map(\|i\| view! {...}).collect_view()` (동적) | `<For each=move \|\| items.get() key=\|i\| i.id …/>` |
| `<div>` (동적 갱신) | `<section role="log" aria-live="polite">` |
| `view! { "Loading..." }` | `view! { "치지직 채팅 연결 중" }` |

## 한국어 처리 주의
- 점자 폭(보통 32셀 = 한국어 ≈ 16자)을 넘기는 한 줄은 의미 단위로 분할.
- 한자·이모티콘은 점자 변환 시 깨질 수 있다. LLM 요약은 한국어 평문으로 강제.
- 숫자·금액은 단위를 한국어로(`10,000원` ✓ / `10000` ✗ — 점자 사용자가 자릿수 카운트하지 않게).
