# autodev 재설계

상태: 설계 초안 · 작성일: 2026-10-03 · 개정: 2026-10-07 · 근거: [9/29 리서치](research/2026-09-29-knowledge-base-and-retrieval.md), [9/30 리서치](research/2026-09-30-knowledge-loop-update.md)

이 문서가 승인되면 저장소의 기존 구현과 문서를 대체한다. 기존 코드는 git 이력에만 남긴다. 이 문서는 구조와 계약을 정하고, 구현 세부는 9절에 단계별로 남긴다.

## 1. 목표와 범위

사용자는 결정만 한다. 개발 중에 생긴 경험과 매일 들어오는 외부 자료가 검증을 거쳐 지식이 되고, 그 지식이 다음 작업의 입력으로 들어간다. Claude Code와 Codex 어느 쪽에서든 같은 파일과 명령으로 동작하고, 상시 서버 없이 Mac이 켜져 있을 때 돈다.

autodev는 이 순환에서 하네스(Agent Host), git, GitHub, Obsidian이 주지 않는 부분만 만든다.

| autodev가 하는 일 | 하지 않는 일 (제공 주체) |
| --- | --- |
| vault의 프론트매터 스키마와 검사(`kb-lint`) | 파일 검색·읽기, 세션 기록 (하네스) |
| 작업 시작 시 지식 브리프 컴파일 | 태스크 실행, PR 생성, 교정, 머지 (각 프로젝트의 기존 흐름) |
| 세션·이슈 경험 수집과 주간 통합 PR | 이력, diff, 리뷰, CI (git, GitHub) |
| 외부 radar의 수집·중복 제거·선별 | 편집기, 속성 UI, 목록 뷰 (Obsidian, Bases) |
| 결정 항목 모으기와 일일 다이제스트 | 스케줄 실행 (launchd, GitHub Actions) |
| 측정, 스킬 평가·승격 | 스킬 로딩·트리거 (하네스) |

### 기존 구현을 버리는 이유

기존 autodev는 계획 리비전 검증(Rust), 이슈 라벨 인증 워크플로, 로컬 delivery 러너와 교정 루프로 구성된 실행 엔진이다. 사용자 목표의 중심은 실행 자동화가 아니라 지식 순환이고, 실제 개발은 프로젝트마다 이미 쓰는 흐름(에이전트 PR, CI, 사람 머지)으로 돈다. 실행 엔진을 유지하면 지식 루프가 그 엔진의 에피소드 형식에 묶이고, 엔진을 쓰지 않는 프로젝트의 경험은 들어오지 못한다. 그래서 실행 엔진을 걷어내고 어떤 프로젝트의 PR과 세션이든 경험 원천으로 읽는다.

대가는 측정이다. 기존 엔진이 남기던 시도 수와 정규화된 실패 서명이 사라지므로 GitHub Actions 실행 기록에서 다시 계산해야 한다(6절).

| 삭제 대상 | 비고 |
| --- | --- |
| `src/`, `Cargo.*`, `tests/`, `test/` | 계획 검증기와 테스트 |
| `scripts/`, `.github/workflows/autodev-*.yml` | delivery 러너, 인증·전이 워크플로 |
| `adr/`, `references/`, `templates/`, `evidence/`, `.autodev/` | 계획·실행 계약과 기록 |
| `docs/` 중 이 문서와 `research/`를 제외한 전부 | 이전 기획 문서 |

## 2. 구성

```text
dev-knowledge (비공개 vault, Obsidian으로 연다)        autodev (도구)
 main 브랜치 (PR로만 변경)                             SKILL.md + references/   모드별 지침
  AGENTS.md        작성 규약과 불변 규칙                 schema/                  프론트매터 JSON Schema
  index.md         생성된 카탈로그                       bin/                     kb-lint, intake-write, capture, metrics, eval, sync
  wiki/            결정된 지식 페이지                    radar/                   수집 스크립트
  consolidations/  통합 PR별 처리 목록                   hooks/                   엔진별 훅
  skills/          승격된 스킬 + ledger.md               launchd/                 일일·주간 작업
  evals/           스킬 평가 정의                        vault-template/          AGENTS.md 원본
  templates/       재사용 파일 자산
  radar/interests.yaml
 intake 브랜치 (스크립트가 직접 push)
  inbox/           후보 (세션, 이슈, radar)
  lookups/         조회 기록
  metrics/         실행별 서명과 주간 표
  radar/           수집·채점 기록
  digest/          일일 다이제스트
```

지식과 도구를 다른 저장소에 두는 이유는 둘이다. 지식 변경과 도구 변경의 리뷰 기준이 다르고, vault는 Obsidian으로 열리므로 도구 코드가 섞이면 안 된다. 설정은 `~/.config/autodev/config.toml`(vault 경로, 대상 프로젝트, 기기 이름, primary 여부)에 둔다.

dev-knowledge는 비공개다. 개인 프로젝트와 업무(SKAX, SKT) 경험이 한 vault에 들어가기 때문이다. 비밀 값은 비공개 저장소에도 남기지 않도록 intake 쓰기 전에 gitleaks를 돌린다.

### main과 intake를 나누는 이유

지식으로 채택된 내용은 사람이 병합한 것만 `main`에 들어와야 한다. 같은 브랜치에 자동 기록까지 직접 push하면 이 규칙을 사후에 탐지할 수밖에 없고, 탐지 전까지 잘못된 페이지를 브리프가 읽는다.

| 브랜치 | 변경 방법 | 내용 |
| --- | --- | --- |
| `main` | PR, 사람 병합. ruleset: PR 필수, 리뷰 스레드 해결 필수, `ci` 필수, 우회 없음 | 결정된 지식, 스킬, 평가, 규약 |
| `intake` | `bin/intake-write`만 push. 쓰기 전에 `kb-lint --intake`와 gitleaks | 후보와 기록 |

`intake`는 `main`에 병합하지 않는다. 주간 통합이 `intake`를 읽고 `main`에 PR을 연다. intake 기록은 쓴 뒤 고치지 않는다. 처리 여부는 `main`의 `consolidations/`가 정본이다(4.3절).

`adoption: accepted`가 `main`에 들어가는 경로는 PR 병합 하나다. 병합하는 사람이 채택 주체이고, 나중에 평가 게이트가 병합을 대신하게 되면 이 주체만 바꾸면 된다. 에이전트는 사용자 계정으로 돌기 때문에 병합을 기술적으로 막지는 않는다. 전역 지침에 "dev-knowledge PR을 병합하지 않는다"를 둔다.

dev-knowledge에는 자동 코드 리뷰 봇을 연결하지 않는다. 봇 스레드도 해결해야 병합할 수 있어서 매주 통합 PR이 막힌다.

여러 기기는 각자 vault를 clone하고 git으로 동기화한다. 하나의 기기만 config에서 `primary = true`이고, 원천 수집·radar 채점·측정·리뷰 반영·다이제스트·주간 통합은 primary에서만 돈다. capture는 세션이 그 기기에 있으므로 모든 기기에서 돈다.

### 커넥터

경험이 들어오는 원천과 결정이 보이는 표면은 커넥터로 붙인다. 어떤 조합이든 config에서 고른다.

| 종류 | 역할 | v1 커넥터 |
| --- | --- | --- |
| 원천 | 경험을 `inbox/` 후보로 만든다 | 세션(훅), GitHub PR·Actions, Linear 이슈, Kaneo 태스크 |
| 표면 | 결정 항목을 보여 주고 알린다 | macOS 알림, GitHub, Linear, Kaneo |

```toml
[[sources]]
kind = "linear"
projects = ["[SKAX]AI PMO", "[SKT] AgentQA"]
participation = ["assignee", "creator"]

[[surfaces]]
kind = "kaneo"
project = "autodev"
```

- 커넥터는 Agent Host의 MCP 연결(Linear, Kaneo)이나 기존 CLI(`gh`)를 쓴다. autodev는 서버를 설치하거나 인증하거나 자격 증명을 저장하지 않는다.
- 커넥터 하나는 `references/connectors/<kind>.md` 한 문서다. 무엇을 읽는지, 어떤 필터를 쓰는지, 항목을 어떻게 만들고 닫는지를 적는다.
- 세션 외의 원천은 primary가 매일 마지막 수집 이후 바뀐 항목을 읽어 `inbox/` 후보로 만든다. PR, 이슈, 태스크 본문은 외부 글이므로 가져오는 단계와 요약하는 단계를 나눈다. 가져오는 단계는 해당 커넥터의 읽기 도구만 허용하고 원문을 그대로 출력하며, 요약은 capture처럼 도구 없이 돌고, 파일 쓰기는 스크립트가 검증한 뒤 한다.
- 업무 경험은 Linear 원천으로만 들어온다. 대상은 위 두 프로젝트에서 사용자가 담당하거나 만든 이슈다. Linear 목록 조회는 댓글 참여로 거를 수 없어 v1은 이 둘로 한정한다.
- 결정의 정본은 vault PR 하나다. 표면은 열린 결정 PR의 링크와 상태를 미러링할 뿐 결정을 받지 않는다. 그래야 표면을 바꾸거나 여러 개 붙여도 결정이 갈라지지 않는다.
- 표면이 닿지 않는 날은 그 표면만 건너뛴다. 다이제스트 파일과 macOS 알림은 항상 남는다.

## 3. 지식 단위와 프론트매터

검색과 인용의 단위는 한 상황을 다루는 H2 절이다. 절은 혼자 읽혀야 하므로 첫 문장에서 주어를 대명사로 받지 않고 다시 쓴다. 페이지는 500줄을 넘지 않고, 한 주제라도 `type`이 다르면 페이지를 나눈다.

본문 절은 상황, 방법, 근거, 주의점, 검증이다. 이 순서는 외부 표준이 아니라 ADR(Context·Decision·Consequences)과 런북 관행을 섞은 이 저장소의 형식이다. 여기에 두 규칙을 더한다.

- `Symptoms` 절에는 오류 메시지와 로그 줄을 코드 스팬으로 그대로 적는다. grep의 검색 훅이 된다.
- `Observed`(이 저장소, 이 날짜, 이 명령 출력)와 `Generalized`(다른 프로젝트에도 적용한다는 해석)를 다른 절로 둔다. 추측이 사실로 굳는 것을 막는다.

```yaml
id: kb-20261005-pnpm-lockfile-v9     # 불변, 파일명과 무관
type: lesson                         # decision | howto | reference | lesson | radar
title: pnpm lockfile v9와 CI 캐시 비호환
description: pnpm 9 lockfile이 8용 캐시 키와 충돌해 CI install이 멈추는 현상과 처리 방법.
applies_when: pnpm 메이저 업그레이드 뒤 CI install 단계가 진행 없이 멈출 때
aliases: [pnpm 잠금파일, ERR_PNPM_LOCKFILE_BREAKING_CHANGE]
tags: [pnpm, ci]
status: stable          # draft | stable | disputed | superseded | deprecated
adoption: accepted      # candidate | accepted | deferred | rejected
revisit_after: null     # deferred일 때 필수
basis: observed         # observed | measured | inferred | assumed
category: gotcha        # lesson만: gotcha | workflow | premise
signature: "install:ERR_PNPM_LOCKFILE_BREAKING_CHANGE:pnpm-lock.yaml"   # lesson, howto. 선택
sources:
  - https://github.com/org/repo/pull/123
  - commit:org/repo@8da5974
  - run:org/repo/1234567890
created: 2026-10-05
updated: 2026-10-05
stale_after: 2027-04-05   # 본문에 "언제 다시 볼지" 문장이 함께 있어야 한다
supersedes: []
superseded_by: null
related: []
```

| 필드 | 이유 |
| --- | --- |
| `description`, `applies_when` | 후보를 고르는 유일한 신호. 무엇인지와 언제 쓰는지를 나눈다 |
| `status` / `adoption` | 내용의 진위와 사람의 결정은 다른 축이다. `rejected`, `deferred`도 `wiki/`에 커밋해야 같은 후보가 다시 올라오지 않는다 |
| `revisit_after` | 보류가 영구 거절과 같아지지 않게 한다. 날짜가 지나면 통합이 다시 후보로 올린다 |
| `basis` | `observed`, `measured`인데 `sources`가 비면 lint 오류. 확인하지 않은 필드는 비운다 |
| `supersedes`, `superseded_by` | 갱신은 편집이 아니라 대체다. 낡은 페이지는 남기고 `superseded`로 표시한다. `superseded_by`는 브리프 후보 조건(아래)을 만족하는 페이지만 가리킬 수 있다. 그렇지 않으면 옛 페이지와 새 페이지가 둘 다 브리프에서 빠진다 |
| `stale_after` + 본문 문장 | 날짜만으로는 모델이 낡은 근거를 버리지 못하고, 언제부터 안 맞는지 적으면 버린다 (2609.31342) |
| `signature` | 6절의 재발 집계와 4.4절의 스킬 승격 조건이 이 문자열로 묶인다 |

`sources`는 영속 참조만 받는다. URL, 커밋, PR, Actions 실행, Linear 이슈 ID다. 세션 ID는 기기 로컬이고 30일 뒤 사라지므로 inbox 노트에만 쓴다.

수치형 신뢰도와 시간 감쇠는 넣지 않는다. 6개월 전 교훈이 재발을 막는 경우가 흔하고, 명시적 대체가 더 정확한 메커니즘이다. `disputed` 페이지는 승자를 고르지 않고 "경쟁 주장" 절에 날짜와 출처가 붙은 주장을 하나씩 두며 각각이 성립하는 조건을 적는다.

### 브리프 후보 조건

유효 상태 없이 쌓기만 하는 메모리는 드리프트 아래에서 메모리가 없을 때보다 나빴다(0.210 대 0.309, 명시 폐기 상태 0.950, 2608.07429). 그래서 브리프와 `index.md`는 `adoption: accepted`이고 `status`가 `stable` 또는 `disputed`이며 `type`이 `radar`가 아닌 페이지만 담는다. 측정(6절)도 같은 조건을 쓴다. 이것이 지켜지지 않으면 위키를 두지 않는 편이 낫다.

### 외부 주장과 확인된 지식

`type: radar` 페이지는 "외부 출처가 이렇게 주장한다"는 기록이다.

- 영속 출처가 하나 이상 있어야 하고, `basis`는 `assumed` 또는 `inferred`만 허용하며, 다른 `type`의 페이지를 `supersedes`할 수 없다.
- `accepted`여도 브리프 후보가 되지 않는다. 채택은 "시험해 볼 가치가 있다"는 결정이다.
- 본문에 `Trial` 절(적용할 프로젝트, 바꿀 것, 확인할 검사)이 있어야 한다.

`accepted` radar 페이지가 병합되면 다음 통합이 `Trial` 절로 그 프로젝트에 이슈 하나를 만든다. 그 뒤는 프로젝트의 평소 개발 흐름이고, 결과는 capture와 원천 수집으로 돌아와 `observed`인 `howto`나 `lesson`이 된다. 외부 방법이 실무 지식이 되는 길은 이것 하나다.

## 4. 순환의 리듬

```text
매 세션      훅 → 세션 큐 (기기 로컬, LLM 호출 없음)
작업 시작    brief: 지식 → 짧은 브리프, lookups/에 기록
매일         capture: 큐의 세션 → inbox/ 후보           (모든 기기)
             sources: PR·Linear·Kaneo 변경 → inbox/ 후보  (primary)
             radar 수집 → 채점                           (Actions → primary)
             metrics 수집, review-sync, digest            (primary)
매주         consolidate: intake → main PR 1건, metrics 표  (primary)
분기         promote: wiki → 스킬 후보 평가 → 스킬 PR      (운영자 세션)
```

경험을 세션마다 위키로 합치지 않는다. 정답 해법만으로 통합해도 이전에 풀던 ARC-AGI 문제 일부의 54%를 다시 틀렸고, 결함은 입력이 아니라 추상화 단계에서 생겼다 (2605.12978). 그래서 추상화는 주 1회, 사람이 보는 PR로만 한다.

일일·주간 작업은 각 기기의 launchd가 `claude -p` 또는 `codex exec`로 autodev 스킬을 부른다. launchd는 상시 서버가 아니며 Mac이 잠들어 있던 동안의 예약은 깨어날 때 한 번 실행된다. 사람이 챙기는 것은 결정 PR을 리뷰하는 일과 분기 승격뿐이다.

autodev가 띄우는 모든 엔진 호출은 `AUTODEV_INTERNAL=1` 환경에서 돌고, 두 엔진의 훅은 이 값이 있으면 큐에 넣지 않는다. 그래야 capture 자신의 세션이 다시 capture되지 않는다.

### 4.1 세션 → inbox (capture)

엔진 훅은 큐(`~/.local/state/autodev/sessions.jsonl`)에 한 줄을 추가하는 일만 한다.

| 엔진 | 훅 | 큐에 남기는 값 |
| --- | --- | --- |
| Claude Code | `SessionEnd` | `session_id`, `cwd`, 시각 |
| Codex | `notify` (`agent-turn-complete`, 턴마다) | `thread-id`, `cwd`, 시각. 같은 thread는 마지막 시각만 갱신 |

훅 안에서 요약하지 않는 이유는 셋이다. Codex에는 세션 종료 이벤트가 없다. 요약 호출 자체가 다시 `SessionEnd`를 일으킨다. `transcript_path`는 비동기로 기록돼 훅 시점에 마지막 메시지가 빠질 수 있다.

`bin/capture`는 마지막 활동 뒤 1시간이 지난 세션을 처리한다. Claude Code는 `claude -p --resume <id> --fork-session --tools "" --strict-mcp-config`로 사용자 세션을 건드리지 않고 도구 없이 분기해서 요약하고, Codex는 `codex exec resume <id>`를 쓴다. 세션에는 저장소 텍스트나 명령 출력처럼 신뢰할 수 없는 내용이 섞여 있으므로 capture는 요약만 출력하고, 파일 쓰기는 스크립트가 출력을 검증한 뒤 한다. Codex는 radar 채점과 같은 조건(5.3절)을 확인하기 전까지 capture도 도구 없는 설정으로 제한한다. 결과가 노트이면 push 뒤에, "기록할 것 없음"이면 그 판정을 큐에 남긴 뒤 완료로 표시한다. 실패만 다음 날 다시 시도한다.

노트는 결정, 교훈, 못 찾은 것을 담고 `status: draft`, `adoption: candidate`다. autodev는 전사 원본을 보관하지 않는다. 세션 JSONL은 형식이 버전마다 바뀌고 Claude Code는 기본 30일 뒤 지운다. 대신 capture는 노트의 `Observed` 절에 명령, 오류 출력, 결정 문장을 요약하지 않고 발췌로 옮기고, 같은 작업의 커밋·PR·Actions 실행을 영속 참조로 연결한다.

대상은 config의 `capture.roots`(기본값 `~/Documents/github/personal/`) 아래 cwd에서 열린 세션이다. 업무 경험은 Linear 원천으로 들어온다(2절).

### 4.2 작업 시작 → 브리프 (brief)

실행 에이전트는 위키를 직접 읽지 않고 브리프만 받는다. 위키를 실행 에이전트에게까지 준 조건이 제안자에게만 준 조건보다 낮았다(63.7% 대 60.9%, 2608.27454). 이 결과는 스킬을 진화시키는 도중의 조건이라 실제 작업 시점에 그대로 옮겨지는지는 확인되지 않았다. 이 분리를 두는 실질적 이유는 컨텍스트 예산을 지키고, "무엇을 몰랐는가"가 세션에 남아 capture의 입력이 되게 하는 것이다.

호출은 작업 에이전트가 한다. 전역 지침 `~/.codex/AGENTS.md`에 "구현 작업은 작업 브랜치를 만든 뒤 autodev brief를 실행하고 시작한다"는 한 줄을 둔다. `~/.claude/CLAUDE.md`가 이 파일의 심볼릭 링크라서 두 엔진이 같은 줄을 읽는다. Claude Code는 v2.1.277부터 프로젝트의 `AGENTS.md`를 직접 읽지만 전역 `AGENTS.md`를 읽는다는 문서는 없으므로 이 링크가 필요하다. 링크가 없는 기기에서는 `~/.claude/CLAUDE.md`에 `@~/.codex/AGENTS.md` 한 줄을 넣는 것이 문서화된 대안이다.

1. 브리프는 별도 프로세스(서브에이전트, 없으면 `claude -p`/`codex exec`)에서 만든다. 입력은 태스크 설명과 현재 저장소·브랜치다.
2. `main` 기준 `index.md`를 읽고 5.1절 절차로 grep하고, 후보의 `status`, `stale_after`, `superseded_by`를 확인한다.
3. 출력은 40줄 이하다. 적용할 방법, 주의점, 확인할 검사, 인용한 페이지 `id`를 담는다. 확신이 없으면 "적용 가능한 지식 없음"이라고 쓴다.
4. 같은 프로세스가 `lookups/`에 조회 기록 파일 하나를 추가한다.

```yaml
lookup_id: 2026-10-05T10:12:00+09:00-mbp
at: 2026-10-05T10:12:00+09:00
repo: syshin0116/foo
branch: fix/pnpm-ci
pr: 17                       # 이미 있으면
vault_sha: 3f2a9c1           # 조회 당시 main
queries: ["ERR_PNPM_LOCKFILE", "lockfile", "잠금파일"]
candidates: [kb-20261005-pnpm-lockfile-v9, kb-20260912-ci-cache]
supplied: [kb-20261005-pnpm-lockfile-v9]   # 브리프에 넣은 페이지
resolution: existed:kb-20261005-pnpm-lockfile-v9   # existed:<id> | none-needed | gap
```

이 기록이 못 찾은 질의 로그, 지식 제공 기록, 인덱스 승격 신호를 함께 맡는다. `supplied`는 브리프에 넣은 페이지일 뿐 실행 에이전트가 따랐다는 뜻이 아니다. `gap`의 확정 결과는 그것을 처리한 통합 PR의 `consolidations/` 파일에 남는다.

### 4.3 주간 통합 (consolidate)

primary 기기의 launchd가 주 1회 `autodev consolidate`를 실행한다. 먼저 `kb-lint --report`로 `stale_after`가 지난 페이지, 30일 넘게 `disputed`인 페이지, `revisit_after`가 지난 `deferred` 페이지, 그리고 결정 뒤 처리가 남은 radar 페이지(`accepted`인데 `Trial` 이슈가 없거나, `rejected`인데 `interests.yaml` 반영 기록이 없는 것)를 뽑는다. 처리 결과(이슈 링크, 반영한 키워드)는 그 통합의 `consolidations/` 파일에 남는다. 입력은 이 목록과, `intake`에서 병합된 어느 `consolidations/` 파일에도 없고 열린 통합 PR에도 들어 있지 않은 모든 항목이다. 출력은 `main`으로의 PR 한 건이다.

- PR은 새 페이지 추가, 기존 페이지 대체, `index.md` 재생성, `radar/interests.yaml` 제안, `consolidations/YYYY-MM-DD.yaml`(페이지마다 처리한 intake 항목과 `gap` 확정 결과)만 담는다. 기존 페이지 본문을 제자리에서 고쳐 쓰지 않는다.
- 에이전트가 페이지마다 최종 `adoption`을 제안하고, 병합이 그것을 확정한다. intake에 쓰는 기록의 `adoption`은 `candidate`뿐이다.
- 사람은 거절이나 보류할 페이지에 리뷰 스레드로 `reject: 이유`나 `defer: 날짜`를 남긴다. primary의 일일 `review-sync`가 그 값을 커밋하고 스레드를 해결한다. 스레드 해결이 필수라서 반영되지 않은 거절이 있는 PR은 병합되지 않는다.
- 대체를 제안한 새 페이지가 거절·보류되면 같은 커밋에서 옛 페이지의 `status`와 `superseded_by` 변경도 되돌린다. 거절된 갱신 때문에 쓰던 지식이 사라지지 않게 한다.
- 같은 `signature`가 두 에피소드 이상에서 나오면 `lesson` 페이지를 제안한다. 기존 페이지와 충돌하는 주장은 `disputed`로 두 주장을 함께 적는다.
- 새 페이지는 PR당 5개 이하이고 그중 radar는 2개 이하다. 경험 후보가 먼저 자리를 받는다. 남는 경험 후보는 다음 주로 넘기고, radar 수집 항목은 14일 동안 뽑히지 않으면 만료된다.
- 처리 여부의 정본은 병합된 `consolidations/` 파일이다. 병합되지 않고 닫힌 PR의 항목은 자동으로 다시 대상이 된다.

### 4.4 분기 스킬 승격 (promote)

다음을 모두 만족하는 `lesson` 또는 `howto`만 스킬 후보가 된다.

1. 같은 `signature`가 페이지 채택 뒤에도 재발했거나, 페이지 없이 2회 이상 반복됐다.
2. 페이지가 `adoption: accepted`다.
3. `sources`에 성공 에피소드와 실패 에피소드가 모두 있다.
4. 후보 SKILL.md가 명령, 검사, 출력 형식을 담고 500줄 이하다. 환경 종속값(절대 경로, 홈 경로, 특정 저장소 파일명)은 경고이고, 버전 핀은 적용 범위 문장이 있을 때만 허용한다.
5. `evals/`의 고정 태스크를 스킬 유무로 두 엔진에서 실행해 개선이 있고, 이전에 통과하던 평가가 두 엔진 모두에서 유지되며, 토큰 증가가 상한(초기값 +30%) 안이다.

후보는 대부분 거부되는 것이 정상이다. 공개 SWE 스킬 49개 중 39개가 통과율 개선 0이었고 토큰은 최대 451% 늘었다 (2603.15401). 4번은 프로젝트 맥락과 어긋난 버전 고정 지침이 성능을 떨어뜨린 사례를 겨냥한다. 5번에서 두 엔진을 다 보는 이유는 절차형 스킬이 엔진을 넘어 잘 전이된 사례(Codex에서 최적화한 스킬로 Claude Code SpreadsheetBench 22.1→81.8, 2605.23904)는 있지만 모든 스킬이 그렇다는 근거는 없기 때문이다.

평가 정의는 항상 `main`의 것을 쓴다. 후보 PR이 `evals/`나 `bin/eval`을 함께 건드리면 CI가 리뷰 전에 거부한다. 자기 개선 시스템 다섯 개 모두에서 평가·기록 경로를 고치는 개조가 발견됐다 (2609.00069).

결정은 `skills/ledger.md`에 한 줄(후보, diff 요약, 평가 표 링크, 결정, 거부 이유)로 남겨 같은 편집이 다시 제안되지 않게 한다. 병합 뒤 같은 `signature`가 재발하거나 평가가 회귀하면 `git revert`하고 장부에 적는다. 승격된 스킬은 `bin/sync`가 두 엔진에 설치한다. `sync`는 매일 vault `main`을 먼저 fast-forward한 뒤 링크를 맞추므로, 브리프도 같은 최신 `main`을 읽는다.

## 5. 검색, radar, 결정

### 5.1 검색 규칙

오늘의 검색은 하네스의 grep·read에 카탈로그 하나와 검색 절차 하나를 얹은 것이다. 소규모 코퍼스에서 파일 탐색 에이전트는 BM25와 같은 수준이었고 BM25가 앞서는 지점은 약 1,000만 토큰이었다 (2607.26497, 리더 모델 하나). 같은 연구에서 원시 탐색 대신 랭킹된 후보 목록을 주자 대규모 정확도가 36.9%에서 69.4%로 올랐다. `index.md`가 그 후보 목록의 값싼 대체물이다.

`index.md`는 한 단계 평면 목록이다. 페이지당 한 줄(링크, `description`, `applies_when`, 태그)이며 `kb-lint`가 생성하고 손으로 고치지 않는다. 주제별 하위 인덱스는 만들지 않는다. 두 번째 라우팅 단계는 도움이 된 적이 없고 정확도를 무너뜨린 경우가 있었다 (2607.17598).

검색 절차는 지금 도구의 한계를 메우는 요령이므로 항상 로드되는 vault `AGENTS.md`가 아니라 브리프만 읽는 `references/brief.md`에 둔다. 도구가 바뀌면 이 파일만 고친다.

- 검색 전에 `index.md`를 읽고 `applies_when:` 줄을 먼저 grep한다.
- 영어 식별자, 오류 문자열, 명령은 `rg -F`로 그대로 친다. 풀어쓰거나 번역하지 않는다.
- 한국어 명사는 조사를 뗀 어간으로, 동사는 가장 짧은 안정 접두로 grep한다. ripgrep은 부분 문자열 매칭이라 `메모리`는 `메모리를`을 잡지만 반대는 안 된다.
- 코드형 토큰이 없는 증상 질의만 한국어·영어 키워드 2~3개와 원인 용어로 확장한다. 어휘가 이미 맞는 질의를 재작성하면 검색이 나빠졌다 (FiQA −9%, 2603.13301).

vault `AGENTS.md`에는 도구가 바뀌어도 유지될 규칙만 둔다. 적용 전에 `status`, `stale_after`, `superseded_by`를 확인한다. 모르면 비운다. 대체된 주장을 고쳐 쓰지 않는다. "지식베이스를 신뢰하라"는 문장은 쓰지 않는다. 문서를 따르라는 지시는 낡은 문서 때문에 답이 뒤집히는 비율을 30~37%에서 66~75%로 키웠다 (2609.31342, Llama·Qwen).

### 5.2 파생 인덱스의 승격 조건

| 인덱스 | 승격 조건 (운영 임계값, 측정 근거 없음) | 승격하는 날의 확인 |
| --- | --- | --- |
| 전문·의미 검색 (QMD 후보) | 한 달간 `resolution: existed`인데 `candidates`에 없던 조회가 10건 이상이거나 전체의 20% 초과 | FTS 토크나이저를 `trigram`으로(기본 `porter unicode61`은 한국어 BM25 0건), 임베더를 한국어 튠 모델로, 질의 경로가 같은 모델을 쓰는지 확인 |
| 그래프 (LadybugDB 임베디드) | 조회 기록에서 다중 홉·비교·시간 질의가 반복되고, 평면 검색이 엔티티는 찾지만 관계를 놓침 | 프론트매터 관계만으로 결정론적 재구축, 기기별 파일, git에 커밋하지 않음 |

두 인덱스 모두 Markdown에서 재생성 가능한 캐시다. 인덱스의 출력은 read 루프에 후보를 공급할 뿐 읽기를 대체하지 않는다. 그래프에서 생성한 텍스트를 검색 단위로 다시 넣지 않는다. LadybugDB는 아카이브된 Kuzu의 MIT 포크로 2026-10-01까지 릴리스가 이어지지만 읽기 전용 재오픈 버그가 열려 있어 승격 시점에 다시 확인한다.

### 5.3 외부 radar

| 조각 | 실행 위치 | 내용 |
| --- | --- | --- |
| 수집 | dev-knowledge의 GitHub Actions, 하루 1회 | arXiv(cs.AI/CL/SE), HF daily papers, HN(points>50), 관심 저장소 릴리스. 마지막 성공 시각부터 수집해 빠진 날을 복구한다. `main`의 `interests.yaml`로 LLM 없이 거르고 정본 ID로 중복을 없앤다 |
| 채점 | primary의 launchd | 점수가 없는 수집 항목을 하루 최대 10건 채점한다. 후보로 올리는 일은 주간 통합이 한다(주 2건) |

2차 게시물(블로그, 트윗, 요약 기사)은 새 후보를 만들지 않고 정본 ID를 가리키는 관찰 기록 파일 하나로 남는다. 관찰 수는 이 파일들을 세서 얻으므로 intake 기록을 고칠 필요가 없다. 이는 중복 방지이지 검증이 아니며, 외부 주장이 실무 지식이 되는 길은 3절의 `Trial` 경로 하나다.

채점은 외부에서 온 글을 모델에 넣으므로 프롬프트 주입을 전제로 한다. 채점 프로세스는 `claude -p --tools "" --strict-mcp-config`로 도구와 MCP 없이 돌고, 정해진 JSON 스키마의 점수만 출력한다. 파일 쓰기는 스크립트가 출력을 검증한 뒤 한다. Codex의 `--sandbox read-only`는 셸 읽기와 MCP 도구를 없애지 않으므로 Codex 채점은 도구가 노출되지 않는 설정을 확인한 뒤에 허용한다.

사람이 `rejected`로 남긴 radar 후보의 사유 키워드는 다음 통합 PR에서 `interests.yaml` 거부 목록 변경으로 제안된다.

### 5.4 결정 인박스

사람의 검토는 예산이다. 위험 명령 차단율이 세션 초반 약 17%에서 프롬프트 50회 이후 약 5%로 떨어졌다 (Anthropic auto mode, 벤더 측정). 그래서 결정은 적은 수의 큰 단위로 모은다.

결정의 입력은 dev-knowledge의 PR 하나뿐이다. 주간 통합 PR(위키 후보와 radar 후보)과 스킬 승격 PR이 그 대상이다. 승인은 병합, 거절과 보류는 리뷰 스레드다. 결정이 `wiki/`의 `adoption`으로 남으므로 닫힌 항목이 다시 올라오지 않는다.

`autodev digest`는 primary에서 하루 한 번 `digest/YYYY-MM-DD.md`를 쓰고 macOS 알림을 보낸다. 결정 항목(열린 결정 PR, intake 쓰기 실패)은 최대 5건이고 넘치면 "외 N건"으로 수만 적는다. 열린 PR은 해결될 때까지 남으므로 다음 날 다시 나온다. 그날 점수가 높은 radar 제목은 결정 항목이 아니라 참고로 붙는다. 각 항목은 "승인할까요?"가 아니라 질문 한 문장, 바뀌는 것, 틀리면 깨지는 것, 링크로 쓴다.

## 6. 측정

측정 대상은 config `projects`에 등록한 저장소의 PR이다. primary의 `bin/metrics`가 매일 끝난 Actions 실행을 읽어 실패 서명을 기록하고, 주 1회 표를 만든다. 메트릭 서버는 없다.

| 지표 | 계산 | 용도 |
| --- | --- | --- |
| 실패 서명 재발 | 서명별 PR 수와 아래 분류 | 확인 지표 |
| PR당 교정 횟수 | 첫 실패 이후 필수 체크가 다시 돈 `head_sha` 수 | 확인 지표 |
| 첫 시도 CI 통과 | PR 첫 `head_sha`의 필수 체크 전체 성공 | 기술 통계 |
| 첫 실행에서 첫 성공까지 | 실행 시각 | 기술 통계 |

실패 서명은 `<job 이름>:<오류 코드 또는 예외 클래스>:<파일>`이고, 실패 job의 로그에서 정해진 패턴으로 뽑는다. 둘째나 셋째 칸을 뽑지 못한 실패는 재발 집계에 넣지 않는다. 무관한 실패가 `test:unknown:unknown` 같은 키로 묶이기 때문이다. 정규화 규칙은 `bin/metrics`에 두고 LLM이 정하지 않는다.

재발은 실패 직전의 lookup(같은 PR이나 브랜치, 실패보다 먼저)을 찾아 그 `vault_sha` 시점의 `main`으로 분류한다. "페이지가 있었다"는 3절의 브리프 후보 조건을 만족한 페이지만 뜻한다.

| 분류 | 판정 | 다음 행동 |
| --- | --- | --- |
| 지식 공백 | 같은 `signature`의 브리프 후보 페이지가 없음 | 통합이 lesson 후보로 |
| 검색 실패 | 페이지가 있었지만 `candidates`에 없음 | 인덱스 승격 신호 |
| 선택 누락 | `candidates`에 있었지만 `supplied`에 없음 | 페이지의 `description`·`applies_when` 재작성 후보 |
| 제공 후 재발 | `supplied`에 있었는데 재발 | 사람이 재검토. 페이지가 틀렸는지, 에이전트가 따르지 않았는지는 세션 노트를 보고 판단 |
| 미조회 | 연결되는 lookup이 없음 | 브리프 호출 누락으로 집계 |

"제공 후 재발"은 페이지 철회나 스킬 승격을 자동으로 일으키지 않는다. 브리프에 넣은 것과 실행 에이전트가 적용한 것은 다르기 때문이다.

이 분류는 한 건으로도 의미가 있다. 통과율로 효과를 확인하지 않는 이유는 쌍대 타깃 111개로도 1.1~4.5점 이득의 신뢰구간이 모두 0을 지났기 때문이다 (2609.23570). 1인 개발자의 한 분기 PR은 수십 건이다. 지식 유무 비교는 채택 페이지가 20개를 넘은 뒤에, 팔 배정을 이슈 번호 홀짝처럼 기계적으로 해서 시작한다. 자기 보고 만족도와 LLM 판정 점수는 지표로 쓰지 않는다.

## 7. 검증 도구

`bin/kb-lint`는 직접 작성한다. 프론트매터 스키마 검사에 쓸 수 있는 유지 중인 패키지가 1인 프로젝트 하나뿐이고, 스키마 밖의 검사가 여러 개 더 필요해 한 파서로 묶는 편이 낫다. Node 22+, `gray-matter`, `ajv`, `ajv-formats`를 쓴다.

| 모드 | 언제 | 검사 |
| --- | --- | --- |
| `main` | PR CI | 스키마(enum, 날짜, `type`별 필수 필드, 빈 `sources`, `deferred`의 `revisit_after`, `wiki/`의 `session:` 근거), 관계(`id` 유일성, 대상 존재, `superseded_by`가 브리프 후보를 가리킴, radar의 대체 금지), `index.md` 재생성 diff 없음, 스킬 PR의 `evals/` 동시 변경 금지 |
| `report` | 통합 시작 | stale, 오래된 disputed, 기한 지난 deferred 목록. 오류가 아니라 통합의 입력 |
| `intake` | 쓰기 직전 | `inbox/`가 모두 `candidate`, lookup 형식, intake 트리 밖 변경 없음, 기존 기록 수정 없음 |

함께 쓰는 기존 도구는 gitleaks(비밀 값), markdownlint-cli2(본문 스타일), lychee `--offline`(Markdown 링크와 앵커)이다. vault `AGENTS.md`는 에이전트에게 상대 경로 Markdown 링크를 쓰게 해서 lychee만으로 에이전트 산출물의 링크를 검증한다. Obsidian 위키링크 검사기(wikilink-check)는 나온 지 며칠 된 1인 프로젝트라 위키링크 파손이 실제로 관찰될 때 도입한다. Obsidian 쪽에는 `stale_after < today()` 필터의 Bases 뷰 하나를 둔다.

## 8. 엔진 중립 경계

| 구성 | 엔진별 차이 |
| --- | --- |
| vault, 스키마, `kb-lint`, `metrics`, radar 수집 | 없음. 파일과 CLI |
| autodev 스킬과 승격된 스킬 | 없음. 두 엔진이 같은 SKILL.md를 읽는다. 설명은 짧게, 트리거 단어를 앞에 둔다 |
| 세션 훅 | `hooks/claude-session-end.sh`, `hooks/codex-notify.sh` |
| 비대화 실행 | `claude -p`/`codex exec`와 resume·fork·도구 제한 인자. `bin/` 공용 함수 하나에서 분기 |
| 스킬 디렉터리 | `bin/sync`의 대상 경로 두 개 |
| 전역 지침 | `~/.codex/AGENTS.md` 하나. Claude Code는 심볼릭 링크나 `@` import로 같은 파일을 읽는다 |

새 엔진을 붙이는 일은 위 표의 아래 네 줄을 추가하는 일이어야 한다. 엔진 고유의 메모리 기능(Claude 자동 메모리, Codex memories)은 기기·엔진별이라 vault와 두 권위로 병존하면 낡은 쪽을 따르는 사례가 있었다. vault `AGENTS.md`가 정본임을 명시하고, 엔진 메모리에는 vault에 있는 내용을 쓰지 않는다.

## 9. 단계

각 단계는 별도 PR이고, 단계마다 Codex 리뷰를 거친다. "구현 때 정할 것"은 리뷰에서 나온 세부로, 그 단계를 구현하며 정한다.

| 단계 | 만드는 것 | 완료 기준 | 구현 때 정할 것 |
| --- | --- | --- | --- |
| 1. 정리와 규약 | autodev: 1절 삭제, README·SKILL.md 골격, `schema/`, `kb-lint`와 테스트, `intake-write`, CI 교체(job 이름 `ci` 유지), 이전 엔진 이슈(#7~#10) 정리. dev-knowledge: 비공개 전환, `intake` 브랜치, `main` ruleset과 CI, vault `AGENTS.md`, 기존 내용 이행, `index.md` 생성 | 두 저장소의 `ci`가 통과하고, 고의로 깨뜨린 프론트매터 다섯 종류와 intake 규칙 위반 두 종류를 `kb-lint`가 각각 잡는다 | 여러 기기의 동시 intake 쓰기(전용 clone, 잠금, rebase 재시도), 건마다 파일 하나인 intake 경로 규칙 |
| 2. 경험 루프 | 훅 두 개, 세션 큐, `capture`, `brief`와 `references/brief.md`, 전역 지침, `sources`와 원천 커넥터(GitHub, Linear), `consolidate`, `review-sync`, `metrics`, launchd 작업 | 개인 프로젝트 작업 2주분이 inbox에 들어오고, 거절 스레드를 반영한 통합 PR 한 건이 병합되며, 측정 표가 두 번 나온다 | 이어진 세션의 증분 capture, 브랜치 생성 전이나 브랜치 이름 재사용 시 lookup과 PR의 연결, 원천 커넥터의 수집 커서를 intake에 영속 저장, 재실행 시도별 로그, 필수 체크 묶음 정의, 로그 보존 기간이 짧은 저장소, intake 쓰기 실패의 다이제스트 전달 |
| 3. radar와 다이제스트 | 수집 워크플로, `interests.yaml`, 도구 없는 채점, `Trial` 이슈 생성, `digest`, 표면 커넥터(macOS, Kaneo, Linear) | 2주 동안 radar 주 2건·결정 5건 상한이 지켜지고, 수집이 하루 빠진 뒤 복구되며, 채점 프로세스에 노출된 도구가 없음을 확인한다 | 수집 기록과 채점 기록의 파일 분리, 만료 처리, `Trial` 이슈 중복 생성 방지(이슈 본문의 radar ID 표식을 먼저 검색), 점수가 나중에 오르는 HN 글을 위한 겹치는 조회 구간 |
| 4. 스킬 승격 | `evals/` 첫 세트, `bin/eval`, `skills/ledger.md`, 경로 거부 CI, `bin/sync` | 평가 세트가 스킬 없이 두 엔진에서 재현 가능하게 돌고, 후보 하나가 채택 또는 거부되어 장부에 남으며, 채택된 경우 두 엔진에서 로드된다 | 대조군 격리(임시 HOME·설정 디렉터리로 설치된 스킬 차단), 평가 에이전트의 쓰기 범위(Claude `--restricted`는 명령 도구를 빼므로 `--tools`로 다시 열어야 함), 검사기 무결성, 스킬 링크 동기화와 revert 반영 |

## 10. 채택하지 않는 것

| 항목 | 이유 |
| --- | --- |
| 공유 Neo4j/Memgraph 서버 | 상시 서버 제약에 걸린다. 여러 기기는 git과 기기별 재구축으로 충분하다. 9/22 계획안과 ADR 초안은 커밋하지 않는다 |
| LLM 추출 엔티티 그래프, 커뮤니티 요약 | 생성 텍스트가 검색 단위가 되면 단일 홉 질의가 무너진다. 재추출할 때마다 그래프가 바뀐다 |
| 임베딩 인덱스 선도입 | 이 규모에서는 grep·read가 충분하다. 조회 기록이 승격 근거가 된다 |
| 세션마다 위키 통합 | 4절 |
| 통과율만 보는 스킬 게이트, 개인 취향 스킬 | 4.4절. 개인 세션에서 뽑은 스킬은 이득이 작고 일관되지 않았다 |
| 세션 JSONL 직접 파싱, 전사 보관 | 형식 불안정이 벤더 문서에 명시돼 있다. 근거는 발췌와 영속 참조로 남긴다 |
| 표면(Kaneo, Linear 등)에서 결정 받기 | 표면마다 결정이 갈라진다. Kaneo 라벨은 태스크별 복사본이라 공유 상태로도 쓸 수 없다 |
| 에이전트 병합을 막는 별도 인증 주체(GitHub App 등) | 1인 운영에 비해 관리 부담이 크다. 지침과 사람 리뷰로 충분하다고 보고, 실제 무단 병합이 관찰되면 다시 검토한다 |
| 엔진 메모리를 지식 저장소로 사용 | 8절 |
| Anthropic Dreaming, Letta sleep-time | 관리형·호스팅이라 git 정본과 맞지 않는다. 배치 통합 + 승인이라는 방향만 따른다 |

## 11. 근거의 강도

설계가 기대는 수치 13개를 원문과 대조했다(2026-10-03).

| 주장 | 확인 결과 | 설계에서의 쓰임 |
| --- | --- | --- |
| 통합이 이전 정답 54% 붕괴 (2605.12978) | 부분 일치. 정답 해법 통합, ARC-AGI 문제 일부. "2배"는 에피소드를 보존한 설정과 강제 통합의 비교 | 방향만 사용 |
| 유효 상태 0.950 대 append-only 0.210 (2608.07429) | 일치 | 3절 |
| grep·read 우위와 1,000만 토큰 교차 (2607.26497) | 일치. 리더 모델 하나, 소규모 구간 차이는 신뢰구간 안 | "충분하다"로만 사용 |
| 낡은 문서와 지시의 효과 (2609.31342) | 일치. 30~37%는 Llama·Qwen 한정 | 3절, 5.1절 |
| 평면 인덱스 효과 (2607.17598) | 부분 일치. Codex에서는 차이 없음. 계층 라우팅의 해는 일치 | 하위 인덱스 금지에만 사용 |
| WikiSkill 절제 63.7 대 60.9 | 일치. 진화 중 조건 | 4.2절에 한계 명시 |
| 스킬 49개 중 39개 이득 0 (2603.15401) | 일치 | 4.4절 |
| 엔진 간 스킬 전이 22.1→81.8 (2605.23904) | 일치. "추론 스킬은 10%만 전이"는 원문에 없음 | 10% 주장은 쓰지 않음 |
| 사람 차단율 17%→5% (Anthropic) | 일치. 벤더 측정, 독립 재현 없음 | 5.4절 |
| 1.1~4.5점, 신뢰구간 0 포함 (2609.23570) | 일치 | 6절 |
| LadybugDB 상태 | 일치. 읽기 전용 재오픈 버그 열림 | 5.2절 |
| 훅 입력값 | 일치. Codex는 턴 이벤트뿐, transcript 기록 지연 | 4.1절 큐 설계 |
| Kaneo 컬럼·라벨 | 부분 일치. 라벨은 태스크별 복사본 | 10절 |

다음 값은 측정 근거가 없는 운영 값이며 첫 분기 기록으로 조정한다. 5.2절의 승격 임계값, 4.3절의 PR당 5페이지와 radar 주 2건, 5.4절의 결정 5건, 4.4절의 토큰 상한 +30%, 4.1절의 1시간 대기.

## 12. 열린 질문과 결정 기록

| 질문 | 막히는 단계 |
| --- | --- |
| 비공개 저장소에서 GitHub ruleset(PR 필수, 스레드 해결 필수)이 현재 요금제로 동작하는지. 안 되면 `main` 보호를 어떻게 대신할지 | 1단계 |
| Kaneo AUT 결정 기록 중 이 설계와 어긋나는 항목(공유 그래프, 실행 엔진 전제) 정정 | 1단계 전 |

결정된 것:

- 2026-10-03: 업무 원천은 Linear `[SKAX]AI PMO`, `[SKT] AgentQA`에서 사용자가 담당하거나 만든 이슈로 한정한다. brief 호출은 전역 지침 한 줄로 한다. 원천과 표면은 커넥터로 골라 붙인다.
- 2026-10-07: 개인과 업무 지식을 dev-knowledge 하나에 두고 저장소를 비공개로 바꾼다. 리뷰 지적 중 구현 세부는 9절의 단계별 목록으로 미루고, 1인 운영에 과한 장치(별도 인증 주체, 로그 보존 경고, 다단계 연결 규칙)는 뺀다.
