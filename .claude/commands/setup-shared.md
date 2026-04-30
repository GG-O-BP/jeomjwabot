---
description: (사문화) shared 크레이트는 이미 존재함. 호출 시 안내만 출력.
allowed-tools: Bash(ls:*)
---

# shared 크레이트 부트스트랩 — 사문화

`shared` 크레이트는 이미 워크스페이스에 존재합니다 (`/shared/`).
현재 노출 타입(2026-04 기준):
- `ChannelId`, `Platform`, `LiveEvent` 계열
- `Settings`, `ChzzkSecrets`, `CimeSecrets`, `CimeAuth`, `ChzzkAuth`, `CimeTokenStatus`, `SecretsPresence`
- `SummaryRequest`, `SummaryResponse`
- `OAuthProgress`, `OAuthStage`, `CIME_DEFAULT_SCOPES`, `CIME_REDIRECT_URI`
- `IpcError`

신규 공용 타입을 추가하려면:

1. `shared/src/<적절한 모듈>.rs`에 struct/enum 추가
2. `shared/src/lib.rs`의 `pub use`에 export 추가
3. `cargo check --workspace --all-targets`로 백/프론트 동시 컴파일 확인

이 커맨드 자체는 더 이상 부트스트랩을 수행하지 않습니다. 호출 시:

```
ls /home/ggobp/Workspace/jeomjwabot/shared
```

위 디렉터리에 파일이 있으면 "shared 크레이트는 이미 존재합니다 — 신규 타입은 직접 추가하세요"라고 안내한 뒤 즉시 종료.
