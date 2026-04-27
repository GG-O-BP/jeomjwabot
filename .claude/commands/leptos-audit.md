---
description: 변경된 Rust 파일을 leptos-reviewer 에이전트로 13대 원칙 감사
allowed-tools: Bash(git diff:*), Bash(git status:*), Bash(rg:*), Bash(ls:*)
argument-hint: (옵션) 감사할 파일 경로들
---

`leptos-reviewer` 서브에이전트를 호출해 감사를 수행한다.

대상 파일:
- `$ARGUMENTS`가 비어있지 않으면 그 파일들.
- 비어있고 git 저장소면 `git diff --name-only HEAD 2>/dev/null` 결과 중 `.rs`만.
- 비어있고 git 없거나 diff 없으면 `src/**.rs` 전체.

서브에이전트의 출력을 그대로 표시하라. 어떤 추가 의견·요약·수정 제안도 붙이지 마라 — 이 커맨드는 감사 보고서만 보여준다. 위배 수정은 사용자가 별도로 요청한다.
