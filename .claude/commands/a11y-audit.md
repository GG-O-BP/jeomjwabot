---
description: 변경된 view! 매크로의 ARIA·점자 적합성을 accessibility-auditor 에이전트로 감사
allowed-tools: Bash(git diff:*), Bash(git status:*), Bash(rg:*), Bash(ls:*)
argument-hint: (옵션) 감사할 파일 경로들
---

`accessibility-auditor` 서브에이전트를 호출해 감사를 수행한다.

대상 파일:
- `$ARGUMENTS`가 있으면 그 파일들.
- 없고 git 저장소면 `git diff --name-only HEAD 2>/dev/null`에서 `view!`를 포함하는 `.rs`만.
- 그 외 `rg -l 'view!\s*\{' src 2>/dev/null` 결과 전체.

서브에이전트 출력을 그대로 표시. 추가 코멘트 금지 — 위배 수정은 별도 요청.
