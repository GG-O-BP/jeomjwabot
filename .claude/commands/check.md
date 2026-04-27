---
description: cargo fmt --check + cargo check + cargo clippy를 한 번에 실행하고 한 줄로 요약
allowed-tools: Bash(cargo fmt:*), Bash(cargo check:*), Bash(cargo clippy:*)
argument-hint: (옵션) 추가 cargo 인자
---

다음 3단계를 순서대로 실행하라. 어느 한 단계가 실패하면 즉시 멈추고 처음 30줄만 출력.

1. `cargo fmt --all -- --check`
2. `cargo check --workspace --all-targets $ARGUMENTS`
3. `cargo clippy --workspace --all-targets $ARGUMENTS -- -D warnings`

마지막에 정확히 한 줄로:
```
fmt: PASS/FAIL · check: PASS/FAIL · clippy: PASS/FAIL · N warnings
```

실패 시 첫 실패 단계의 출력을 30줄까지만 보여주고, 명백한 단순 수정(미사용 import, 누락된 `;` 등)이 보이면 패치를 제안하라 (적용은 사용자 승인 후).
