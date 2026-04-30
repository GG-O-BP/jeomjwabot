---
description: README.md와 README-braille.md의 동기화 상태를 점검 (헤딩 구조·수정 시각·길이 비교)
allowed-tools: Bash(git log:*), Bash(grep:*), Bash(wc:*), Bash(diff:*), Bash(stat:*), Bash(test:*)
---

점좌봇은 1차 사용자가 점자 단말기 사용자다. 모든 의미 변경은 `README.md`와 `README-braille.md` 둘 다에 반영돼야 한다. 이 커맨드는 두 파일이 어긋나 있는지 점검한다.

## 점검 절차

1. **헤딩 구조 비교** — 두 파일의 `^##` / `^###` 라인 개수가 같은지.
   ```
   grep -cE '^##' README.md
   grep -cE '^##' README-braille.md
   grep -cE '^###' README.md
   grep -cE '^###' README-braille.md
   ```

2. **마지막 git 수정 시각 비교** — README.md가 더 최근에 수정됐다면 점역본이 뒤처진 신호.
   ```
   git log -1 --format=%ct README.md 2>/dev/null
   git log -1 --format=%ct README-braille.md 2>/dev/null
   ```

3. **현재 파일시스템 mtime 비교** — 미커밋 변경 감지.
   ```
   stat -c %Y README.md
   stat -c %Y README-braille.md
   ```

4. **본문 라인 수 비교** — 한 쪽이 통째로 누락된 섹션을 가진 경우 감지.
   ```
   wc -l README.md README-braille.md
   ```

5. **링크 카운트 비교** — `]( ` 패턴 개수가 양쪽 비슷한지(완전 일치는 어렵지만 큰 격차는 누락 의심).
   ```
   grep -cE '\]\(' README.md
   grep -cE '\]\(' README-braille.md
   ```

## 출력 양식

각 지표별 한 줄 요약 + 이상 감지 시 "DRIFT" 표시. 마지막에 한 줄 종합:

```
### Braille sync report

- ## headings:  README=N₁  Braille=N₂  [OK | DRIFT]
- ### headings: README=M₁  Braille=M₂  [OK | DRIFT]
- last commit:  README=…  Braille=…    [OK | README newer by Δs]
- last mtime:   README=…  Braille=…    [OK | README newer by Δs]
- line counts:  README=L₁  Braille=L₂   (참고용)
- markdown links: README=K₁ Braille=K₂  [OK | DRIFT > 20%]

Verdict: SYNCED / NEEDS BRAILLE UPDATE / NEEDS README UPDATE
```

DRIFT가 1개라도 있으면 Verdict는 `NEEDS BRAILLE UPDATE` (또는 역방향). 누락된 섹션 추정이 가능하면 어떤 헤딩이 차이인지 grep으로 추출해 1~3개만 노출.

추가 의견·잡담 금지. 검증 보고서만 출력.
