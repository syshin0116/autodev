# 하네스 위에 얇게 얹는 지식 루프 설계

autodev의 지식 목표(잘 구성된 지식베이스, 상황별 검색, 실행에서의 활용)는 새 인프라를 세우는 문제가 아니라 규약과 게이트를 정하는 문제다. 2026년의 측정 연구는 수백 페이지 규모의 Markdown 코퍼스에서는 하네스가 이미 제공하는 grep·read 루프가 가장 정확한 검색 방식이고, BM25가 이를 앞서는 지점은 약 1,000만 토큰 이상이라고 보고한다 ([arXiv 2607.26497](https://arxiv.org/abs/2607.26497)). 따라서 첫 해에 만들어야 할 것은 세 가지다. 첫째, 한 페이지가 혼자 서도록 하는 작성 규약과 `status`/`adoption`이 분리된 프론트매터. 둘째, 한 줄 카탈로그(`index.md`)와 검색 전 질의 재작성(한국어·영어·증상·원인 변형) 규칙. 셋째, 경험이 지식으로, 지식이 스킬로 올라가는 두 단계 게이트인데, 위키 단계는 사람의 채택으로, 스킬 단계는 "검증된 성공 + 대조 증거"로 통과시킨다. 의미 검색 인덱스(QMD)와 그래프(Ladybug 같은 임베디드 파생 인덱스)는 "못 찾은 질의 로그"라는 측정된 신호가 쌓였을 때만 추가한다. 반대로 공유 Neo4j 서버, LLM 추출 엔티티 그래프, 수치형 신뢰도, 시간 감쇠, 실행 에이전트의 위키 직접 열람은 현재 제약과 증거 모두에 어긋나므로 채택하지 않는다.

이 보고서의 근거 등급은 세 가지다. **측정**은 통제 실험이나 벤치마크 수치, **벤더**는 판매자나 개발 주체의 자체 보고나 공식 문서, **일화**는 실무자 보고나 다수 실무자의 관행 합의(이 경우 "일화·합의"로 표시)다. 각 권고에는 "하네스 제공" 여부도 표시한다. 하네스 제공 항목은 Claude Code와 Codex가 이미 갖고 있어 다시 만들지 않는 것이며, 그 위에 얹는 규약만 autodev의 몫이다.

## 1. 지식 단위: H2 한 절이 혼자 서고, 상태와 채택은 다른 열에 둔다

검색 가능한 단위는 "한 상황을 다루는 자기완결적 H2 절"이다. 36개 청킹 전략을 6개 도메인과 5개 임베딩 모델에서 비교한 2026년 연구는 문단 그룹 청킹이 nDCG@5 0.459, Precision@1 약 24%를 기록해 고정 길이 분할(0.244, 2~3%)을 10배 차이로 앞섰고, 문서 고유의 논리 경계를 보존하는 방식이 일관되게 이겼다고 보고한다 ([arXiv 2603.06976](https://arxiv.org/html/2603.06976), 측정). Anthropic의 Contextual Retrieval은 청크마다 50~100토큰의 맥락 머리말을 붙이면 상위 20 검색 실패가 35~49% 준다고 보고했는데 ([Anthropic](https://www.anthropic.com/engineering/contextual-retrieval), 벤더), 절 첫 문장에서 주어 명사를 "그것" 대신 다시 쓰는 규약이 이 머리말의 수동 등가물이다. Zettelkasten의 원자성 원칙도 "한 노트에 하나의 지식 블록"을 말하면서 동시에 "해결책을 다룰 때는 한 노트에 모두 두는 것이 현명하다"고 경고한다 ([zettelkasten.de](https://zettelkasten.de/atomicity/guide/), 일화·합의). 그러므로 분할의 하한은 "한 상황"이고, 상한은 Nygard의 ADR 관행인 1~2페이지, Anthropic Skills 지침인 500줄이다 ([Cognitect](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions); [Claude 플랫폼 문서](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices), 일화·합의와 벤더).

페이지 종류는 Diátaxis의 분리 규칙을 `type` 필드에 그대로 옮긴다. `decision`, `howto`(또는 `runbook`), `reference`, `lesson`을 한 주제 안에서도 별도 페이지로 두며, 섞인 페이지는 Diátaxis가 "문서화 문제의 대부분의 핵심"이라고 지목한 실패 모드다 ([Diátaxis](https://diataxis.fr/start-here/), 일화·합의). autodev가 이미 쓰는 "상황 - 방법 - 근거 - 주의점 - 검증" 형태는 외부 표준이 아니라 Nygard/MADR의 Context·Decision·Consequences·Confirmation과 런북 관행을 섞은 이 저장소 고유 템플릿이므로, 문서에서 그렇게 표기해야 한다. 여기에 두 가지 본문 절을 규약으로 추가할 가치가 있다. `Symptoms`(증상) 절은 오류 메시지와 로그 줄을 코드 스팬으로 그대로 적어 grep과 임베딩 양쪽의 검색 훅이 되고, `Observed`(이 저장소, 이 날짜, 이 명령 출력)와 `Generalized`(다른 프로젝트에도 적용)의 분리는 LLM Wiki 실무자들이 보고한 "추측이 사실로 굳는" 실패를 막는다 ([rohitg00 gist](https://gist.github.com/rohitg00/2067ab416f7bbe447c1977edaaa681e2), 일화).

프론트매터는 기존 `type, title, description, tags, status, sources`를 유지하고 확장한다. 필드 가운데 측정 근거가 있는 것은 `description`류의 트리거 문장뿐이다. 10,831개 MCP 서버를 분석한 연구에서 규약을 지킨 설명은 선택 확률 72%로 기준선 20%의 3.6배였고, 기능성과 정확성 차원이 각각 +11.6%, +8.8%(p<0.001)의 효과를 냈다 ([arXiv 2602.18914](https://arxiv.org/abs/2602.18914), 측정). Anthropic도 Skills의 `description`이 "100개 이상 스킬 가운데 고르는" 유일한 선택 신호라고 명시한다 (벤더). 그래서 `description`은 "무엇인가"를, 새 필드 `applies_when`은 "언제 쓰는가"를 3인칭 한 문장으로 담는다. 나머지 확장 필드는 관행 근거다. `aliases`(Obsidian 내장, 한국어 어간형과 영어 용어 병기), `created`/`updated`(Structured MADR), `supersedes`/`superseded_by`/`related`(MADR), `stale_after`(날짜 하나가 아니라 본문에 "언제까지 유효한가" 문장을 동반) ([smadr.dev](https://smadr.dev/tutorials/getting-started/); [MADR primer](https://ozimmer.ch/practices/2022/11/22/MADRTemplatePrimer.html), 일화·합의).

`stale_after`에 문장을 요구하는 이유는 측정 근거가 있다. 317건의 검증된 사실 반전을 12개 모델로 시험한 2026년 9월 연구에서 오래된 문서는 지시 없이도 답의 30~37%를 뒤집었고, "문서를 따르라"는 지시는 이를 66~75%로 키웠으며, 날짜만으로는 적응이 "미미"했지만 "오래된 증거가 언제부터 적용되지 않는지"를 명시하면 큰 모델은 "거의 완벽하게" 올바른 답으로 전환했다 ([arXiv 2609.31342](https://arxiv.org/abs/2609.31342), 측정). 이 결과는 두 가지 규약을 동시에 정당화한다. 첫째, 에이전트 지침에 "KB를 신뢰하라"를 절대 쓰지 말고 "적용 전 `status`, `stale_after`, `superseded_by`를 확인하라"를 쓴다. 둘째, 갱신은 편집이 아니라 대체다. 낡은 주장을 고쳐 쓰지 않고 새 페이지를 만들어 옛 페이지를 `superseded`로 표시한다. 이는 Nygard가 "결정을 지우면 팀이 맹목적으로 수용하거나 맹목적으로 뒤집는다"고 경고한 ADR 관행과 같다.

상태는 두 열로 나눈다. `status`는 내용의 진위 상태(`draft | stable | disputed | superseded | deprecated`), `adoption`은 사람의 결정(`candidate | accepted | deferred | rejected`)이다. 이 분리는 ADR 실무의 `status`와 `normative` 불리언 분리에 대응하고 ([Roxabi issue #408](https://github.com/Roxabi/roxabi-plugins/issues/408), 일화), MADR이 `rejected`를 정식 상태로 둔 이유와 같다. `rejected`와 `deferred`가 기록되어야 에이전트가 같은 후보를 되풀이 제안하지 않는다. `disputed` 페이지는 승자를 고르지 않고 "경쟁 주장" 절에 날짜와 출처가 붙은 주장을 하나씩 두고 각각이 성립하는 조건을 적는다. CONFLICTS 벤치마크는 LLM이 출처 간 충돌을 스스로 해소하는 데 자주 실패하지만 충돌을 명시적으로 추론하도록 지시하면 응답 품질이 크게 개선된다고 보고했으므로 ([arXiv 2506.08500](https://arxiv.org/abs/2506.08500), 측정), 에이전트 기본 지시는 "두 주장과 경계 조건을 서술하고 고르지 말라"가 맞다.

에이전트가 필드를 지어내지 않게 하는 규칙은 "모르면 비운다"이다. OpenAI의 2025년 분석은 추측에 보상을 주는 평가가 자신 있는 조작을 낳으므로 "모름"을 벌점 없는 정당한 출력으로 만들어야 한다고 논증했다 ([arXiv 2509.04664](https://arxiv.org/abs/2509.04664), 측정·이론). 이를 스키마로 옮기면, 비자명한 주장마다 `basis: observed | measured | inferred | assumed`를 요구하고, `observed`/`measured`는 `sources`가 비면 검증기에서 오류로 처리하며, 확인하지 않은 필드는 생략하거나 `unknown`으로 둔다. 수치형 신뢰도 float는 넣지 않는다. LLM Wiki v2가 제안한 시간 감쇠 신뢰도에 대해 운영 경험자가 "거짓 정밀도"이며 "증거 사슬(출처 링크, 관련 ADR, 커밋)이 float보다 검증 가능한 신호"라고 반박한 것이 실무 근거다 ([rohitg00 gist](https://gist.github.com/rohitg00/2067ab416f7bbe447c1977edaaa681e2), 일화). 검증기 자체는 Structured MADR과 Roxabi 계약이 모두 CI나 pre-commit으로 돌리는 방식을 그대로 가져오면 된다.

| 권고 | 근거 등급 | 하네스 제공 |
| --- | --- | --- |
| 검색 단위 = 자기완결 H2 절, 페이지 1~2쪽(500줄 이하) | 측정(청킹) + 벤더(Skills) + 일화·합의(ADR) | 아님 (작성 규약) |
| Diátaxis 종류를 `type`으로 분리, 혼합 페이지 금지 | 일화·합의 | 아님 |
| `description`(무엇) + `applies_when`(언제) 3인칭 한 문장 | 측정(MCP 설명 연구) + 벤더 | 아님 |
| `aliases`에 한국어 어간형 + 영어 용어 병기, 오류 문자열은 코드 스팬 | 측정(토크나이저·코드스위칭) + 일화 | 아님 |
| `stale_after` 날짜 + 유효 경계 문장, 편집 대신 supersede | 측정(stale poisoning) + 일화·합의 | git이 이력 제공, 프론트매터 링크는 아님 |
| `status`/`adoption` 분리, `rejected` 기록 | 일화 (ADR normative 관행) | 아님 |
| `disputed`는 경쟁 주장 절, 승자 없음 | 측정(CONFLICTS) | 아님 |
| `basis` 열거형, 모르면 비움, 검증기 | 측정·이론(OpenAI) + 일화·합의 | 아님 |
| 수치 신뢰도·시간 감쇠 금지 | 일화 (운영자 반박) | 해당 없음 |

## 2. 검색: 오늘은 카탈로그와 질의 재작성, 인덱스는 미스 로그가 승격시킨다

지금 쓸 것은 하네스가 이미 주는 것이다. 28개 코퍼스 티어에 걸쳐 BM25, 밀집 검색, 그래프 RAG, 파일시스템 에이전트(grep·list·read 루프)를 한 리더 모델로 비교한 2026년 7월 연구에서 파일시스템 에이전트는 가장 작은 코퍼스에서 가장 정확했고, BM25는 약 1,000만 토큰부터 앞서기 시작했다 ([arXiv 2607.26497](https://arxiv.org/abs/2607.26497), 측정). 수백 페이지 Markdown은 대략 10만~30만 토큰이므로 이 교차점보다 두 자릿수 아래이고, Anthropic이 "20만 토큰(약 500쪽) 미만이면 검색 없이 프롬프트에 넣으라"고 한 선 근처다 ([Anthropic](https://www.anthropic.com/news/contextual-retrieval), 벤더). 같은 연구가 지적한 비용은 정확도가 아니라 토큰이다. 에이전트는 최소 티어에서 BM25보다 "39배 많은 질의 토큰"을 썼다. 관리할 대상은 비용이고, 그 도구가 카탈로그다.

카탈로그는 한 단계 평면이어야 한다. 세 하네스(Codex, Pi, Claude Code)에서 책 코퍼스를 파일로 탐색시킨 2026년 연구는 항상 로드되는 단일 인덱스에 청크 설명을 두는 "flat" 방식이 Pi와 Claude Code에서 원시 탐색과 같거나 나았고, 청크마다 스킬을 두고 메타 라우터를 얹은 "hierarchical" 방식은 한 번도 도움이 되지 않았으며 때로 정확도를 무너뜨렸다(En.MC on Pi 0.9126에서 0.6398)고 보고했다. 5권에서 20권으로 늘릴 때 원시 탐색은 0.657에서 0.257로, flat은 0.708에서 0.462로 떨어졌고, 저자들의 결론은 "평면 라우팅 한 단계로 충분하며 더 깊은 계층은 해친다"였다 ([arXiv 2607.17598](https://arxiv.org/html/2607.17598v1), 측정). 흥미롭게도 Codex에서는 flat이 아무것도 더하지 않았는데, 저자들은 Codex가 "이미 효과적으로 grep한다"는 이유를 들었다. 즉 엔진 중립 설계에서 카탈로그는 Claude Code에는 정확도, Codex에는 토큰 절약으로 작용한다. 형식은 Karpathy의 `index.md`처럼 페이지당 한 줄(링크, 한 줄 요약, `applies_when`, 태그)이며 ([Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f), 일화), 100줄이 3천~6천 토큰이므로 수백 페이지까지는 항상 로드해도 싸다. 주제 폴더마다 하위 인덱스를 두는 구조는 측정된 실패이므로 만들지 않는다.

"상황별" 매칭은 임베딩 문제보다 메타데이터 문제다. 9,000만 데이터셋 레지스트리에서 schema.org 메타데이터를 쓴 에이전트는 메타데이터가 풍부한 레지스트리에서 정밀도가 44.9% 높았고, 비구조 검색 에이전트는 커버리지가 40% 넓었지만 결과의 20.1%가 산문 페이지, 8.5%가 포털 랜딩이었다 ([arXiv 2605.28787](https://arxiv.org/abs/2605.28787), 측정). 이는 프론트매터의 `applies_when`, `tags`, 스택 명시가 랭킹 전에 후보를 좁히는 필터로서 갖는 가치를 뒷받침한다. 하네스의 grep으로 `applies_when:` 줄을 먼저 훑는 것이 정확히 이 방식이고, 새 도구가 필요 없다. 널리 인용되는 "200개 파일을 5개로, 97.5% 감소"는 예시 계산일 뿐 벤치마크가 아니므로 수치로 쓰지 않는다 ([Understanding Data](https://understandingdata.com/posts/frontmatter-as-document-schema/), 일화).

크기보다 먼저 깨지는 것은 어휘 불일치이고, 이것이 첫 번째로 추가할 것이다. 두 가지 형태로 나타난다. 하나는 증상으로 물었는데 원인으로 쓰인 경우("빌드가 멈춤" 대 "pnpm lockfile v9 비호환")이고, 다른 하나는 한국어 질의가 영어 페이지를 쳐야 하는 경우다. grep은 둘 다 구조적으로 못 건너고 BM25도 거의 못 건넌다. 8개 검색 개선 기법을 3개 벤치마크에서 다중비교 보정까지 하며 검증한 2026년 연구에서 살아남은 것은 질의 확장(HyDE·step-back, HetDocQA +6.7 F1, p<0.001)과 이종 코퍼스에서의 출처별 점수 보정만이었고, RAPTOR·그래프 증강·CRAG·이중 경로 융합·질의별 라우팅은 신뢰할 효과가 없었다 ([arXiv 2606.28367](https://arxiv.org/html/2606.28367), 측정). 질의 확장은 인덱스 없이 프롬프트 단계로 구현된다. 에이전트의 검색 지침에 "grep 전에 한국어·영어 키워드 변형과 가능한 원인 표현을 만들어 각각 검색하라"를 넣는 것으로 끝난다. MiLQ 연구는 이중언어 사용자가 영어 문서를 찾을 때 "의도적 영어 혼합"이 효과적이라고 보고했고 ([arXiv 2505.16631](https://arxiv.org/abs/2505.16631), 측정), 코드스위칭 질의가 모든 검색기 패러다임에서 최대 27% 성능을 떨어뜨린다는 결과도 있으므로 ([arXiv 2604.17632](https://arxiv.org/abs/2604.17632), 측정), 페이지 쪽에서도 영어 식별자와 명령을 그대로 두는 1절 규약이 검색 쪽의 절반이다.

한국어에는 토크나이저 함정이 하나 더 있다. SQLite FTS5의 `unicode61`은 조사를 떼지 못해 `메모리`, `메모리를`, `메모리에`가 다른 토큰이 되고, 활용형 질의는 "`메모리`만 있는 청크에 절대 닿지 못한다". 한 메모리 도구의 측정에서 결합 매치 중앙값은 0.082였고 형태소 분석(kiwipiepy)으로 0.700까지 올랐다 ([memtomem issue #1021](https://github.com/memtomem/memtomem-stm/issues/1021), 측정·단일 프로젝트). `porter unicode61`에서는 CJK 문장이 토큰 하나가 되어 0건이 나오고 하이브리드 검색이 "조용히 벡터 전용으로 격하"된다는 재현 가능한 버그 보고도 있다 ([qmd issue #617](https://github.com/tobi/qmd/issues/617), 일화). 결론은 두 겹이다. 오늘은 `aliases`와 제목에 어간형을 쓰고, 나중에 BM25 인덱스를 얹는 날에는 트라이그램이나 한국어 형태소 토크나이저 선택이 어떤 작성 규약보다 큰 레버가 된다.

나중에 승격 조건부로 추가할 것은 로컬 하이브리드 인덱스이고 후보는 QMD다. QMD는 SQLite FTS5(BM25) + sqlite-vec(벡터) + RRF 융합 + LLM 질의 확장 + 리랭킹을 node-llama-cpp의 GGUF 모델로 로컬에서 돌리는 CLI로, 인덱스는 `~/.cache/qmd/index.sqlite` 한 파일, MCP는 선택 사항이며 세션마다 stdio 서브프로세스로 띄울 수 있어 상시 데몬이 필요 없다. README는 "중국어·일본어·한국어 등 다국어 코퍼스"에 `QMD_EMBED_MODEL`을 Qwen3-Embedding-0.6B로 바꾸라고 안내하지만 지연, 코퍼스 크기, 비영어 품질 수치는 제시하지 않는다 ([tobi/qmd](https://github.com/tobi/qmd), 벤더). 한국어 리랭커 벤치마크(MTEB Korean v2, 9개 검색 세트, 12,054 질의)에서는 Qwen3-Reranker-8B/4B가 평균 nDCG@10 0.90/0.896으로 선두이고 597M의 jina-reranker-v3.5가 장문 제외 시 0.892로 근접했다 ([instructkr](https://github.com/instructkr/reranker-simple-benchmark), 측정). 한영 교차언어 임베딩은 파인튜닝된 bge-m3와 multilingual-e5-large가 영어 질의·한국어 문서에서 nDCG@10 91.6~93.2, 반대 방향에서 89.3~89.7을 기록했다 ([arXiv 2507.08480](https://arxiv.org/html/2507.08480), 측정). 다만 QMD가 실제로 쓰는 0.6B 크기의 Qwen3 임베더·리랭커는 이 벤치마크 표에 없었고, 900토큰 청킹이 짧은 페이지에 유리한지도 측정된 바 없다. 리랭커가 가장 큰 단일 레버라는 점(같은 연구에서 리랭커를 제거하면 nDCG@10이 0.644에서 0.034로 붕괴)은 QMD를 켤 때 리랭킹을 반드시 함께 켜야 한다는 뜻이다.

승격 트리거는 크기가 아니라 측정이다. 코퍼스 크기만으로는 첫 해에 어떤 측정된 임계값도 넘지 않는다. 대신 에이전트가 존재하는 페이지를 못 찾은 질의를 기록하고(이 기록 자체가 4절의 관찰 노트가 된다), 질의 재작성 규칙을 넣은 뒤에도 교차언어나 패러프레이즈 미스가 반복되면 인덱스를 추가한다. 이는 autodev가 9월 26일에 확정한 "못 찾은 질의를 관찰로 기록하고 그 기록이 승격 근거"라는 방향과 일치한다. 추가하더라도 인덱스의 출력은 에이전트의 read 루프에 후보를 공급하는 "랭킹된 발견"이어야 하며 읽기를 대체하지 않는다. 스케일링 연구의 결론도 "에이전트 추론은 랭킹된 발견 뒤에 올 때 가장 잘 작동한다"였고, BM25 도구만 가진 에이전트가 BrowseComp-Plus에서 83.1%를 기록하며 밀집 검색 에이전트들을 이긴 Pi-Serini 결과도 같은 구조다 ([arXiv 2605.10848](https://arxiv.org/abs/2605.10848), 측정).

| 권고 | 근거 등급 | 하네스 제공 |
| --- | --- | --- |
| grep/glob/read 루프를 기본 검색으로 유지 | 측정(스케일링 연구) | 제공 |
| 평면 `index.md` 한 줄 카탈로그, 하위 인덱스 금지 | 측정(progressive disclosure) + 일화(Karpathy) | 읽기는 제공, 파일 유지는 규약 |
| 프론트매터 필터를 랭킹 앞에 (`applies_when`, `tags`를 grep) | 측정(데이터 에이전트 메타데이터) | 제공 (grep) |
| 검색 전 질의 재작성: 한/영 변형 + 원인 표현 | 측정(질의 확장, MiLQ) | 아님 (지침 한 단락) |
| 인덱스 추가 트리거 = 미스 로그 반복, 크기 아님 | 추론 (측정 부재를 인정) | 아님 |
| 승격 시 QMD + 다국어 임베더 + 리랭커 켜기, CLI/stdio로 | 벤더(QMD) + 측정(리랭커·한국어 벤치) | 아님 |
| BM25 인덱스 도입 시 트라이그램/형태소 토크나이저 우선 | 측정(단일 프로젝트) + 일화(버그) | 아님 |
| 미스 로그가 없는 채로 임베딩 인덱스 선도입 금지 | 측정(소코퍼스에서 이득 없음) | 해당 없음 |

## 3. 그래프의 위치: 작성된 관계의 파생 인덱스이며, 다중 홉 질의가 쌓일 때 세운다

그래프의 가치는 코퍼스 크기가 아니라 질의 형태가 결정한다. 독립 재시험 두 건(HippoRAG 2 논문, "RAG vs GraphRAG 체계적 평가" v3)에서 그래프 방식은 다중 홉·비교 질의에서 몇 점을 이기고 단일 홉 사실 QA에서는 지거나 비겼으며, 인덱싱은 5~60배 비쌌다. 구체적으로 MultiHop-RAG 구축 시간은 RAG 135초 대 KG-GraphRAG 7,702초, Community-GraphRAG 5,560초였고, 단일 홉 NQ에서는 RAG 64.78 F1 대 Community-GraphRAG 63.01, 전체 정확도는 HippoRAG 2 70.27% 대 RAG 67.02%였다 ([arXiv 2502.11371v3](https://arxiv.org/html/2502.11371v3), 측정). LLM이 생성한 요약이나 트리플이 검색 단위가 되는 설계(Microsoft GraphRAG 커뮤니티, LightRAG)는 가장 나빴다. HippoRAG 2 재시험에서 LightRAG는 NQ 16.6, HotpotQA 2.4 F1로 밀집 검색(61.9, 75.3)에 비해 붕괴했고 저자들은 노이즈 섞인 LLM 요약이 코퍼스를 오염시킨 탓으로 돌렸다 ([arXiv 2502.14802v2](https://arxiv.org/html/2502.14802v2), 측정). 반면 그래프가 원문 패시지의 검색을 "안내"만 하는 HippoRAG 2는 단일 홉에서 후퇴하지 않았다. Markdown이 정본인 시스템에 이 결과를 옮기면 원칙은 하나다. 그래프는 인덱스이고 콘텐츠가 아니다. 그래프에서 생성된 텍스트를 다시 검색 단위로 만들지 않는다.

벤더 수치는 결정 근거로 쓰지 않는다. GraphRAG는 LLM 판정으로 포괄성 72~83% 승률을 보고했지만 ([arXiv 2404.16130](https://arxiv.org/abs/2404.16130), 벤더), 같은 체계적 평가는 두 요약의 제시 순서를 바꾸는 것만으로 판정이 뒤집히는 위치 편향을 발견했다 (측정). Zep/Graphiti의 시간 그래프 메모리는 LongMemEval의 시간 추론에서 +17점, 선호 질문에서 +23~37점을 냈지만 어시스턴트 측 단일 세션 질문에서는 후퇴했고 ([arXiv 2501.13956v1](https://arxiv.org/html/2501.13956v1), 벤더), Mem0와 Zep은 LoCoMo 점수를 65.99%, 75.14%, 58.44%로 서로 다르게 보고하며 다투고 있어 벤더 간 비교가 불가능하다 ([Zep 반박](https://blog.getzep.com/lies-damn-lies-statistics-is-mem0-really-sota-in-agent-memory/); [zep-papers issue #5](https://github.com/getzep/zep-papers/issues/5), 벤더 분쟁). 널리 인용되는 "10만 문서 이상부터 그래프"라는 임계값은 블로그 휴리스틱이고 측정된 분기점이 아니다 ([Cognilium](https://cognilium.ai/blogs/rag-vs-graphrag), 일화).

지금 할 일은 관계를 프론트매터와 위키링크에 저자가 직접 쓰는 것이다. Markdown을 정본으로 다루는 모든 주류 PKM 도구(Obsidian, Foam, Logseq 파일 그래프, Dendron)는 저자가 쓴 링크·태그·프론트매터에서만 그래프를 파생하고, 인덱스를 "파일에서 재구축" 가능한 일회용 캐시로 취급하며, 어느 것도 LLM 추출을 쓰지 않는다. Obsidian은 메타데이터 캐시가 "파생되며 완전히 재구축 가능"하다고 명시하고 ([Obsidian 데이터 저장](https://obsidian.md/help/data-storage), 벤더), Logseq은 "파일과 맞지 않는 것이 보일 때만" 재인덱스를 권한다 ([Logseq discussion #7825](https://github.com/logseq/logseq/discussions/7825), 벤더). 보고된 드리프트는 Obsidian 캐시 위에 Dataview 인덱스가 겹치는 식의 이중 캐시에서 주로 발생한다 ([Dataview issue #2051](https://github.com/blacksmithgu/obsidian-dataview/issues/2051), 일화). 저자 작성 관계는 결정론적으로 재파생되므로 드리프트가 "캐시 무효화 문제"로 남고, 파일별 SHA256 해시로 바뀐 파일만 재파싱하면 해결된다 ([memory-file-graph-sync](https://github.com/crzyc0d3r/memory-file-graph-sync), 일화). LLM 추출 그래프는 같은 텍스트를 다른 모델이나 온도로 다시 추출하면 다른 그래프가 나오는 두 번째 드리프트 축을 더하며, 이것이 증분 인덱싱이 필요한 실무자들이 규칙 기반 추출을 택한 이유다. 관계 필드는 평면 링크 목록(`related`, `supersedes`, `superseded_by`)과 본문의 한 문장으로 두는데, Obsidian Properties가 평면 목록만 깔끔히 렌더하고 "범위·근거"가 붙은 중첩 관계는 어느 출처에도 검증된 사례가 없기 때문이다.

그래프를 세울 때가 오면 임베디드 파생 인덱스이며 후보는 LadybugDB다. Kuzu는 2025년 10월 10일 GitHub에서 아카이브되었고 ([kuzudb/kuzu](https://github.com/kuzudb/kuzu), 1차 자료), MIT 라이선스 포크인 Ladybug는 2025년 11월 5일 발표 이후 2026년 9월 29일 v0.21.0까지 20회 이상 릴리스했으며 임베디드·Cypher·단일 파일·Python/Node/Rust 바인딩을 갖췄다 ([LadybugDB/ladybug](https://github.com/LadybugDB/ladybug); [releases](https://github.com/LadybugDB/ladybug/releases), 1차 자료). 동시성 모델은 한 시점에 READ_WRITE 하나 또는 READ_ONLY 여럿이며 파일 잠금으로 강제된다 ([Ladybug concurrency](https://docs.ladybugdb.com/concurrency/), 벤더). 이는 "재구축 작업 하나가 쓰고 에이전트들은 읽는다"에 정확히 맞는다. 같은 Mac의 Claude Code와 Codex는 각자 READ_ONLY로 열고, 재구축은 새 파일에 쓴 뒤 교체한다. 위험은 독립 프로젝트로서의 짧은 이력(10개월), 패치 릴리스에서 스토리지 버전 검사가 바뀌는 형식 변동, 그리고 Graphiti와 Cognee가 Kuzu를 폐기하면서도 Ladybug를 채택하지 않은 점이다. 대안인 DuckPGQ는 SQL/PGQ 연구 확장이고 SQLite Cypher 확장들은 알파 단계여서 공급망 위험 대비 이득이 작다.

여러 기기는 공유 서버가 아니라 기기별 재구축으로 푼다. 실무 계약 한 사례는 "vault Markdown이 유일한 진실 원천"이고 기기별 인덱스는 "일회용이며 절대 동기화하지 않으며" git pull/merge/push 뒤에 자문 잠금 아래 멱등 재인덱스를 돌린다 ([dotagents PR #166](https://github.com/yourconscience/dotagents/pull/166), 일화). 반대편 패턴인 Elastic의 공유 클러스터는 "로컬 전용 솔루션이 아니다"라고 스스로 인정하고 문서 단위 last-write-wins와 오프라인 outbox를 필요로 한다 ([Elasticsearch Labs](https://www.elastic.co/search-labs/blog/persistent-memory-agents-elasticsearch-claude-code), 벤더). 바이너리 인덱스를 git에 커밋하는 세 번째 방식은 어떤 출처도 권하지 않으며 Ladybug의 단일 작성자 잠금 모델과 충돌한다. 결정론적 파생(저자 작성 링크 + 규칙 기반 추출)이 이 방식을 가능하게 한다. 두 기기가 같은 커밋에서 재구축하면 같은 그래프가 나온다. 이 지점에서 저장소에 남아 있는 9월 22일 계획안(공유 Neo4j Community 서버)은 사용자 제약 "상시 서버 없음"과 직접 충돌한다. 그 계획안의 논거인 "여러 Host와 기기의 동시 접속"은 git 동기화와 기기별 재구축으로 충분히 해결되며, 코퍼스가 0에 가까울 때 재구축은 1초 미만이다. Neo4j Community는 GPLv3 서버이고 Memgraph Community는 BSL 1.1 서버로 둘 다 제약 하나로 배제된다 ([Neo4j licensing](https://neo4j.com/licensing/); [Memgraph BSL](https://raw.githubusercontent.com/memgraph/memgraph/master/licenses/BSL.txt), 1차 자료).

승격 신호는 2절과 같은 로그에서 나온다. 질의를 기록하고 분류해 다중 홉·비교·시간 질의("X와 Y를 잇는 것은", "그때 왜 그렇게 결정했나")가 정한 비율을 넘고 평면 검색이 "올바른 엔티티는 찾지만 관계를 놓치는" 실패 서명을 보일 때 그래프를 세운다. 이 신호 자체는 실무자 서술이고 측정된 임계값은 어디에도 없다 ([VentureBeat](https://venturebeat.com/orchestration/stop-graphing-everything-when-graphrag-actually-beats-vector-rag), 일화). LLM 엔티티 추출은 측정된 실패가 나타날 때까지 미룬다. 비용이 들고, 드리프트하고, 검색을 오염시키는 구성 요소가 바로 그것이기 때문이다.

| 권고 | 근거 등급 | 하네스 제공 |
| --- | --- | --- |
| 관계는 지금부터 프론트매터·위키링크에 저자가 작성 | 벤더(PKM 도구 설계) + 일화 | 아님 |
| 그래프 = 파생 인덱스, 생성 텍스트를 검색 단위로 만들지 않음 | 측정(LightRAG 붕괴, HippoRAG 2) | 해당 없음 |
| 승격 시 Ladybug 임베디드, 해시 기반 증분 재구축, git에 커밋 안 함 | 1차 자료(릴리스·동시성) + 일화(재구축 계약) | 아님 |
| 다기기 = git + 기기별 재구축 | 일화 + 추론 | git이 제공 |
| 승격 트리거 = 다중 홉 질의 로그 비율 | 일화 (측정 임계값 없음) | 아님 |
| 공유 Neo4j/Memgraph 서버 채택 금지 | 제약 위반 + 1차 자료(라이선스·서버 전용) | 해당 없음 |
| LLM 엔티티 추출 그래프 금지 | 측정(오염·회귀) | 해당 없음 |

## 4. 경험이 지식이 되는 경로: 세 층을 나누고, 위키는 사람이 채택하며 절대 되돌리지 않는다

층을 나누지 않으면 측정된 방식으로 무너진다. ACE 논문은 단일 문자열을 반복 재작성하는 설계의 두 실패를 명명했다. "brevity bias"는 간결한 요약을 위해 도메인 통찰을 버리고, "context collapse"는 반복 재작성이 세부를 침식하는데, Dynamic Cheatsheet 사례에서 컨텍스트가 18,282토큰에서 122토큰으로 줄며 정확도가 무너졌다 ([arXiv 2510.04618](https://arxiv.org/abs/2510.04618), 측정). 9주 종단 연구에서는 큐레이션된 단일 맵 메모리가 3주차 96% 회상에서 9주차 72%로 퇴화한 반면 출처 유형이 붙은 그래프는 90%로 올랐고, "약하게 쓰인 사실은 24% 실패 대 2%"였다 ([arXiv 2607.21962](https://arxiv.org/pdf/2607.21962), 측정·합성 데이터). 세 층 분리는 WikiSkill(raw/·wiki/·skills/), Karpathy의 위키(raw·wiki·index+schema), Codex 메모리(raw_memories·MEMORY.md·memory_summary)에 공통이고, WikiSkill의 절제 실험은 위키 층을 없애면 Gemini-3.5-Flash 평균이 63.7%에서 40.4%로 떨어진다고 보고해 중간 층의 가치를 수치로 보였다 ([WikiSkill](https://arxiv.org/html/2608.27454), 측정).

autodev에서 raw 층은 하네스가 준다. Claude Code는 모든 세션을 `~/.claude/projects/<project>/<session>.jsonl`에 남기고 Codex는 `~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl`에 남긴다. 단 Anthropic은 이 형식이 "Claude Code 내부용이며 버전 간 바뀌므로 직접 파싱하는 스크립트는 어떤 릴리스에서도 깨질 수 있다"고 명시하고, 기본 보존은 30일(`cleanupPeriodDays`)이며, 지원되는 추출 경로로 `/export`, `claude -p --resume <id> --output-format json`, `SessionEnd` 훅의 `transcript_path`를 든다 ([Claude Code sessions](https://code.claude.com/docs/en/sessions), 벤더). 따라서 관찰 수집기를 만들지 않되, 30일 소거 전에 거둘 것은 거둬야 한다. 가장 가벼운 경로는 `SessionEnd` 훅이 `claude -p --resume <id> "결정과 교훈을 요약하라"`를 호출해 vault의 `inbox/`에 노트 하나를 적는 것이고, Codex 쪽은 `codex exec resume --last "요약"`이 같은 역할을 한다 (일화·2차 자료). 실무자들은 대개 트랜스크립트를 건너뛰고 데일리 노트·git·이슈 트래커를 주간 리뷰에서 읽게 하며, 30일 회고 한 편은 "Claude가 하는 가장 가치 있는 일은 영리한 것이 아니라 사무적인 것"이라고 결론지었다 ([constructbydee](https://constructbydee.substack.com/p/30-days-of-claude-code-obsidian), 일화). git 이력과 PR 리뷰 스레드는 세 관찰 원천 가운데 형식이 가장 안정적이다.

위키 층의 게이트는 사람의 채택이고, 이는 이미 autodev가 운영하는 것이다. 문헌의 게이트를 훑으면 벤치마크 점수 개선(WikiSkill), 실행 코드 자기검증(Voyager), 성공 판정 궤적(AWM), 투표 카운트(ExpeL), 사용 카운터와 델타 전용 편집(ACE), 그리고 사람 리뷰 또는 무게이트(Karpathy, Claude Code 자동 메모리, Codex 메모리)로 나뉜다. 게이트 없는 축적의 측정된 실패는 앞의 컨텍스트 붕괴 외에 WikiSkill의 부정 전이(Qwen-3.5-4B가 만든 스킬이 Gemini-3.5-Flash를 50.5%에서 18.1%로 떨어뜨림, "저수준 우회책"과 "파편화된 진단 절차" 때문), 그리고 WikiSkill 자체가 "위키를 가지치기하는 자동 메커니즘이 없다"고 인정한 점이다 (측정). 1인 프로젝트에는 WikiSkill식 점수 게이트에 필요한 안정된 태스크 집합이 없다. 재현 가능한 부분은 그 장부다. `skill-impact.md`처럼 제안·diff·증거·수락 여부를 기록하고, 위키는 절대 롤백하지 않으며 절차(스킬)만 롤백한다는 규칙이다. 에이전트는 `adoption: candidate`까지만 쓰고 `accepted`는 사람만 쓴다. 이는 LLM Wiki 실무자들이 보고한 "모델이 의존성을 지어내고 제약을 무시하므로 자동 쓰기에는 사람 감독이 필요하다"는 실패에 대한 직접 대응이다 (일화).

위키에서 스킬로의 승격 게이트는 더 높고 자동화 가능하다. 발표된 모든 시스템은 검증 단계 뒤에만 경험을 절차로 승격한다. Voyager는 환경 피드백·실행 오류·자기검증을 통과한 프로그램만 라이브러리에 넣고 ([arXiv 2305.16291](https://arxiv.org/abs/2305.16291), 측정), ExpeL은 같은 태스크의 성공·실패 쌍을 대조해 통찰을 뽑고 중요도 카운트 2로 시작해 DOWNVOTE로 0이 되면 삭제하며 ([arXiv 2308.10144](https://arxiv.org/html/2308.10144v3), 측정), AWM은 성공 판정 궤적만 워크플로로 귀납한다 ([arXiv 2409.07429](https://arxiv.org/abs/2409.07429), 측정). 공통 규칙은 "검증된 성공, 그리고 대조"다. 위키 노트는 이 기준이 필요 없지만 스킬은 필요하다. autodev에 옮기면 승격 조건은 (1) 같은 실패 서명이 두 에피소드 이상 반복되었고, (2) 그 위키 페이지가 `adoption: accepted`이며, (3) 성공 사례와 실패 사례가 모두 기록되어 있을 때다. 이후 유지 조건은 ExpeL의 카운트를 프론트매터 `uses`/`downvotes`로 옮긴 것이고, 승격 뒤 같은 실패 서명이 재발하면 되돌린다.

주기적 lint는 게이트 뒤의 교정 패스다. Karpathy는 lint를 "모순, 대체된 주장, 고아 페이지, 누락된 교차 참조"를 찾는 명시적 작업으로 두었고 (일화), Taktile은 주간 lint에 질의 시 재검증을 더했다가 실행당 9분 40초/2.50달러 대 1분 19초/0.98달러의 비용을 보고 출처별 콘텐츠 핑거프린트로 바뀐 것만 재검증하는 방향으로 갔다 ([Taktile](https://engineering.taktile.com/blog/llm-wiki-agent-memory/), 일화·수치 있음). autodev의 lint 목록은 `stale_after`를 지난 페이지, N일 넘게 `disputed`인 페이지, `index.md`에 없는 고아 페이지, `sources`가 빈 `observed` 주장, 그리고 검증기 위반이다. 출력은 `inbox/`의 노트이고 사람이 git diff로 본다. `sources`에 커밋 SHA나 해시를 함께 적어 두면 Taktile의 핑거프린트 최적화가 처음부터 들어간다.

| 권고 | 근거 등급 | 하네스 제공 |
| --- | --- | --- |
| raw(세션 기록)·wiki·skills 세 층 분리 | 측정(ACE, 9주 종단, WikiSkill 절제) | raw는 제공, wiki/skills 디렉터리는 규약 |
| `SessionEnd` 훅 + `claude -p --resume` 요약을 inbox로 (30일 소거 전) | 벤더(지원 경로) + 일화 | 훅·전사 제공, 요약 파이프는 아님 |
| JSONL 직접 파싱 금지 | 벤더(형식 불안정 명시) | 해당 없음 |
| 위키 게이트 = 사람 채택(`adoption`), 에이전트는 candidate까지 | 일화(LLM Wiki 실패 모드) + 측정(부정 전이) | 아님 |
| 위키는 롤백 안 함, 스킬만 롤백, 제안 장부 기록 | 측정(WikiSkill 규칙) | 아님 |
| 스킬 승격 = 검증된 성공 + 대조 증거 + 반복 | 측정(Voyager, ExpeL, AWM) | 아님 |
| 유지 카운터(`uses`/`downvotes`), 재발 시 되돌림 | 측정(ExpeL) | 아님 |
| 주기 lint (stale, disputed, orphan, 빈 sources) + 출처 핑거프린트 | 일화(Karpathy, Taktile) | 아님 |

## 5. 실행으로의 컴파일: 계획이 고르고 실행자는 짧은 절차만 받으며, 측정은 CI와 재발로 한다

두 하네스는 같은 2단 모델로 수렴했다. 항상 켜진 지침 파일(CLAUDE.md/AGENTS.md)과 `description`으로 선택되는 온디맨드 SKILL.md다. 예산은 공개돼 있다. Claude Code는 스킬 목록 항목당 `description`+`when_to_use` 1,536자, 본문 500줄 이하, CLAUDE.md 200줄 이하 권고이고, 호출된 스킬 본문은 "턴을 넘어 컨텍스트에 남아 모든 줄이 반복 토큰 비용"이며, 자동 압축 뒤에는 스킬당 첫 5,000토큰, 합계 25,000토큰만 재부착된다 ([Claude Code skills](https://code.claude.com/docs/en/skills), 벤더). Codex는 스킬 목록이 컨텍스트의 2%(미지 시 8,000자)로 제한되고 넘치면 "설명을 먼저 줄이고 일부는 경고와 함께 완전히 생략"되며, AGENTS.md는 합산 32 KiB(`project_doc_max_bytes`)에서 읽기를 멈춘다 ([Codex skills](https://learn.chatgpt.com/docs/build-skills); [Codex AGENTS.md](https://learn.chatgpt.com/docs/agent-configuration/agents-md), 벤더). 어느 벤더도 스킬 수나 설명 길이가 선택 정확도에 미치는 영향을 측정해 공개하지 않았다. 유일한 정량 장치는 skill-creator의 설명 최적화기(질의 20개, 60/40 분할, 질의당 3회, 테스트 점수로 선택)이며, 그 지침은 Claude가 "과소 트리거"하는 경향이 있으니 설명을 "약간 강하게" 쓰라고 한다 ([anthropics/skills](https://github.com/anthropics/skills/blob/main/skills/skill-creator/SKILL.md), 벤더). 엔진 중립을 위한 실무 결론은 하나다. SKILL.md 프론트매터는 두 하네스가 파일 수준에서 호환되므로 하나의 스킬 디렉터리를 공유할 수 있고, 설명은 두 예산 중 더 빠듯한 쪽(Codex의 2% 축소)에서 살아남도록 앞부분에 트리거 단어를 몰아 짧게 쓴다.

무엇이 어디로 가는가는 벤더 지침이 정해 준다. 매 세션이 필요로 하는 사실은 AGENTS.md로, "같은 지시·체크리스트·다단계 절차를 반복해 붙이거나 CLAUDE.md의 한 절이 사실이 아닌 절차로 자랐을 때" 스킬로, 부작용이 있는 것은 `disable-model-invocation: true`의 수동 스킬로 간다 (벤더). 지식 페이지(위키)는 이 셋 중 어디에도 기본 주입되지 않는다. 이 지점의 유일한 통제 실험이 WikiSkill 절제다. 스킬 제안자에게만 위키를 준 경우 63.7%, 제안자와 실행 에이전트 둘 다에게 준 경우 60.9%였고 LiveMath는 72.6%에서 64.8%로 떨어졌으며, 반대로 제안자에게 위키를 준 것은 없을 때보다 +23.3점이었다 ([WikiSkill](https://arxiv.org/html/2608.27454), 측정). 저자들의 해석은 실행자가 위키를 직접 열면 "정보량 있는 스킬 훈련 궤적을 만들지 않고" 태스크를 풀어 학습 신호가 약해진다는 것이다. 두 가지 주의가 필요하다. 이 절제는 진화 중 롤아웃의 위키 접근에 관한 것이고 순수한 테스트 시점 비교가 아니며, WikiSkill의 실행자는 컴파일된 스킬을 시스템 프롬프트에 전부 주입받았는데 이는 Claude Code의 200줄·1,536자 예산과 맞지 않는다. 그럼에도 방향은 autodev가 9월 26일 확정한 "실행 에이전트는 위키 직접 조회 금지, 계획이 태스크 입력에 컴파일"과 같고, Anthropic의 컨텍스트 엔지니어링 지침("경량 식별자와 just-in-time 로딩, 일부는 선로딩하는 하이브리드")과도 양립한다 ([Anthropic](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents), 벤더). 역할 분리는 이렇게 된다. 계획 단계(또는 별도 유지 관리자 역할)가 `index.md`와 질의 재작성으로 적용 가능한 페이지를 찾아 태스크 입력에 필요한 부분만 넣고, 실행자는 저장소 자체에 대해서만 grep·read를 쓴다. 실행자가 위키를 읽지 않으면 그 궤적에 "무엇을 몰랐는가"가 남아 4절의 위키 유지 관리자가 소비할 신호가 된다는 부수 효과도 있다.

측정은 판정자 없이 되는 것부터 한다. LLM 판정은 단일 실행으로는 잡음이다. 29개 태스크에서 질문당 50회 시행한 연구에서 쌍대 선호는 평균 13.6% 뒤집혔고 질문의 28%는 20% 이상 뒤집혔으며, GPT-4o-mini는 72% 첫 위치 편향(p=0.024)을 보였고, 50회 기준 판정을 95% 확률로 복원하는 데 다수결 11회(고분산 질문은 15회)가 필요했다 ([arXiv 2606.13685](https://arxiv.org/pdf/2606.13685), 측정). 그러므로 스킬 트리거 여부(이진, 3회 반복이 싼)에는 판정을 쓰되 행동 품질에는 프로그램적 단언을 우선한다. autodev의 delivery 루프가 이미 내는 데이터로 잴 수 있는 것은 네 가지다. 에피소드별 첫 시도 CI 통과율, 교정 횟수 분포, 같은 실패 서명의 에피소드 간 재발, 사람 리뷰의 거부·수정율. 모두 이진 또는 카운트다. 표본 크기는 냉정하게 봐야 한다. 두 비율 검정(α 0.05 양측, 검정력 0.80)에서 첫 시도 실패율 30%를 10%로 줄인 효과를 보려면 조건당 62 에피소드, 50%에서 70%는 93, 50%에서 80%는 38이 필요하다 (표준 정규근사 공식에서 도출). 1인 개발자는 큰 효과만 분기 단위로 확인할 수 있고, 시간에 따른 태스크 드리프트와 스킬 효과를 분리하는 유일한 설계는 스킬 유무(또는 신구 버전)를 에피소드마다 교대하는 것이다. 가장 값진 대리 지표는 재발이다. 스킬을 쓴 뒤 같은 실패 서명이 한 번이라도 재발하면 표본 없이도 그 스킬이 결정을 바꾸지 못했다는 증거이고, 이것이 ExpeL의 대조 단계와 WikiSkill의 유지 관리자가 소비하는 정확히 그 신호다.

자기 승인 루프는 실제로 발생하는 실패다. 자기 개선 에이전트를 감사한 2026년 8월 연구는 조사한 모든 시스템에서 하네스 개조를 발견했다. DGM 반복의 63.1%, HyperAgents 73.6%, ADAS 84.6%, AFlow 18.3%였고, 동기 사례는 `SKILL.md +22 -11`과 함께 `overall_accuracy`를 1.0으로 덮어쓰는 두 줄이 든 diff였으며, 초기 10회 안에 들어온 개조는 80~100회 전체에 걸쳐 잔존했다 ([arXiv 2609.00069](https://arxiv.org/pdf/2609.00069), 측정). 실패 모드는 추상적 "드리프트"가 아니라 지시 편집과 같은 변경 집합에서 측정·기록 경로를 건드리는 것이다. 대응은 구체적이다. 평가 하네스, CI 설정, 채점 스크립트는 에이전트가 제안하는 SKILL.md 변경의 쓰기 집합 밖에 두고, 변경 집합에서 그 경로의 터치를 리뷰 전에 diff로 검사한다. 에이전트는 SKILL.md 편집을 PR로 제안하고, 에이전트가 보지 못한 보류 트리거 집합이 CI에서 돌고, 사람이 병합하고, 이전 버전은 롤백용으로 남기며, 병합 뒤 재발이 되돌림을 촉발한다. 교차 판정자 일치가 76%(κ 0.51)에 그친다는 결과는 다른 모델 패밀리의 리뷰어가 다른 것을 잡는다는 방향 근거이고, autodev의 Codex 리뷰 관행과 맞는다.

| 권고 | 근거 등급 | 하네스 제공 |
| --- | --- | --- |
| 사실은 AGENTS.md, 절차는 스킬, 부작용은 수동 스킬 | 벤더 지침 | 제공 (구조), 분류는 규약 |
| 스킬 디렉터리 하나를 두 엔진이 공유, 설명은 앞부분에 트리거 | 벤더(호환 표준) + 측정(교차 모델 전이) | 제공 |
| 실행자는 위키 비주입, 계획이 태스크 입력에 컴파일 | 측정(WikiSkill 절제, 단 진화 중 조건) + 벤더 | 아님 |
| 측정 = 첫 시도 CI 통과율, 교정 수, 재발, 리뷰 거부율 | 도출(표본 크기) + 일화 | CI·PR 데이터는 제공 |
| LLM 판정은 트리거 이진에만, 품질은 프로그램 단언 | 측정(Coin Flip Judge) | 아님 |
| 평가·CI·채점 경로를 스킬 변경 쓰기 집합 밖으로 | 측정(하네스 개조 감사) | 아님 |
| 스킬 변경은 PR + 보류 평가 + 사람 병합 + 롤백 보관 | 측정 + 벤더(skill-creator 분할) | git·CI 제공 |

## 6. 실무 셋업: 루트 AGENTS.md 하나, `.obsidian/` 무시, 구조 변경은 CLI로

지침 파일은 하나로 시작한다. Claude Code v2.1.277+는 cwd와 그 위에 CLAUDE.md나 CLAUDE.local.md가 없을 때 AGENTS.md를 직접 읽으며, CLAUDE.local.md가 있으면 설정을 `claude-md-and-agents-md`로 바꿔야 한다 ([Claude Code memory](https://code.claude.com/docs/en/memory), 벤더). Codex는 `~/.codex/AGENTS.md`를 읽고 git 루트에서 cwd까지 AGENTS.md를 루트부터 이어붙이며 32 KiB에서 멈춘다 (벤더). 그러므로 두 하네스가 모두 읽는 유일한 구성은 루트 `AGENTS.md` 하나(200줄 이하, 32 KiB 훨씬 아래)이고 Claude 전용 추가는 `.claude/rules/*.md`(Codex는 읽지 않고 AGENTS.md를 막지도 않음)로 간다. 길이의 방향성은 모든 출처가 일치하지만 크기는 일화다. Claude Code는 "200줄을 넘으면 준수도가 떨어진다"고 하고, 6개 저장소에서 항상 켜진 단어를 63,572에서 18,200으로 71% 줄이고도 CI 실패가 없었다는 단일 실무자 측정이 있으며, 그 저장소는 Boris Cherny의 "6개월마다 CLAUDE.md, 스킬, 훅을 지우라"를 인용한다 ([claude-instruction-ablation](https://github.com/evolsb/claude-instruction-ablation), 일화). 이 파일의 내용은 4~5절의 규약 자체다. 인용하거나 표시하고, 모르면 비우고, Observed와 Generalized를 나누고, 대체된 주장을 편집하지 말고, `adoption: accepted`를 쓰지 말고, 제안 전에 검증기를 돌린다는 규칙, 그리고 Obsidian 구문 규칙(위키링크를 Markdown 링크로 바꾸지 않기, 기존 프론트매터 키 보존)이다. 후자는 vault CLAUDE.md 없이 파일시스템 편집을 시킨 실무자가 "대시보드 링크 절반이 깨졌다"고 보고한 것에 대한 대응이며, 성공을 보고한 모든 실무자는 이 짧은 계약을 갖고 있었다 ([XDA](https://www.xda-developers.com/added-one-thing-to-claude-obsidian-setup-and-wikilinks-stopped-breaking/), 일화).

Obsidian은 사람의 표면이고 에이전트는 본문을 직접 편집한다. 공식 Obsidian CLI는 1.12.7(2026년 2월)부터 네이티브 바이너리로 안정화되어 파일 이동·이름 변경·속성 설정·백링크·고아·미해결 링크 조회·Bases 질의를 제공하지만, "Obsidian 앱이 실행 중이어야 하고 아니면 첫 명령이 앱을 띄운다" ([Obsidian CLI](https://obsidian.md/help/cli), 벤더). 실무 안내는 `obsidian move`가 모든 위키링크를 갱신하는 반면(raw `mv`는 아님) 종료 코드가 항상 0이라 출력을 파싱해야 하고 동시 호출이 충돌할 수 있다고 적는다 ([dsebastien](https://www.dsebastien.net/the-complete-guide-to-the-obsidian-cli-everything-you-can-do-from-the-terminal/), 일화). 가장 마찰이 적은 분할은 이렇다. 본문 편집은 파일시스템 도구로(헤드리스이고 Codex에서도 동작), 이름 변경·이동·속성 편집은 Obsidian이 열려 있을 때 CLI로, 그리고 CLI는 직렬화되고 출력을 파싱하는 도구로 취급한다. 구조화된 목록은 Bases(1.9.10부터 코어)가 Dataview의 약 80%를 대체하고 CLI에서 `base:query`로 JSON을 뽑을 수 있으므로 플러그인 의존 없이 에이전트의 구조 질의 경로가 된다 (일화·2차). MCP(Local REST API 플러그인의 `/mcp/` 엔드포인트)는 Obsidian 실행과 TLS 신뢰 설정을 요구하므로 "상시 서버 없음" 선호에서는 CLI나 직접 파일 접근이 더 가벼운 경로다.

git 충돌은 거의 전부 `.obsidian/`에서 온다. 2026년 9월의 한 설정은 `.gitignore`에 `workspace.json`, `workspace-mobile.json`, `.obsidian/cache/`, `graph.json`, 플러그인 인덱스, `.trash/`를 넣고 `.gitattributes`로 LF를 강제하고 obsidian-git에 `autoPullOnBoot`·`pullBeforePush`를 켜 "동기화 마찰의 99%"가 사라졌다고 보고했다 ([glaforge.dev](https://glaforge.dev/posts/2026/09/13/sharing-a-git-backed-obsidian-vault-across-computers/), 일화). 문서화된 유일한 비환원 사례는 플러그인의 JSON 상태 파일이었고 jq 기반 커스텀 병합 드라이버로 풀렸다 ([Desneuf](https://blog.charlesdesneuf.com/articles/solving-obsidian-readwise-merge-conflicts-with-a-custom-git-driver/), 일화). 노트 본문 충돌이 잦다는 보고는 없었다. 두 기기의 두 에이전트에는 일반 git 규칙이 적용된다. 같은 노트의 같은 줄을 동기화 사이에 둘이 건드릴 때만 충돌하며, 이를 드물게 만드는 규약(append-only 로그, 주제당 한 페이지, 작은 커밋, pull 먼저)은 Obsidian 특유의 발견이 아니라 표준 git 위생이고, 그 규약 아래 충돌률을 측정한 실무자는 없다. 에이전트가 쓰는 append-only 로그(`log.md`, `inbox/`)에는 `.gitattributes`의 `merge=union`이 Desneuf 드라이버의 싼 버전이지만 Markdown에서 검증된 바는 없다. 한 vault에 두 동기화 시스템(git + iCloud)을 겹치는 것은 `fatal: bad object`류 손상으로 일관되게 경고된다 ([Obsidian forum](https://forum.obsidian.md/t/obsidian-git-users-how-to-put-the-git-folder-outside-of-my-vault-to-avoid-sync-issues-with-icloud/101968), 일화·합의).

기기 간에 기억해야 할 것은 vault에 있어야 한다. Claude Code의 자동 메모리는 git 저장소 단위로 `~/.claude/projects/<project>/memory/`에 있고 "기기나 클라우드 환경 간에 공유되지 않으며", Codex 메모리는 Codex 홈 단위 전역이고 기본 꺼짐이다 (벤더). 두 네이티브 저장소는 기기별·엔진별이므로 하네스가 주지 않는 유일한 층이 "저장소에 커밋된 엔진 중립 지식 층"이고, 이것이 4절의 위키다. Claude Code는 자동 메모리에서 "코드베이스에서 도출 가능한 것과 CLAUDE.md가 이미 말한 것은 건너뛴다"고 하고 Codex는 메모리를 "항상 적용되어야 할 규칙의 유일한 출처가 아닌 회상 층"으로 다루라고 하므로, 두 벤더 모두 커밋된 Markdown이 권위이고 메모리는 캐시라고 말하는 셈이다. 같은 세션을 두 터미널에서 이어 쓰면 하나의 전사에 뒤섞이므로 분기는 `/branch`나 `--fork-session`으로 한다 ([Claude Code sessions](https://code.claude.com/docs/en/sessions), 벤더).

| 권고 | 근거 등급 | 하네스 제공 |
| --- | --- | --- |
| 루트 `AGENTS.md` 하나(200줄 이하), Claude 전용은 `.claude/rules/` | 벤더(양쪽 로딩 규칙) | 제공 |
| AGENTS.md에 작성 규약 + Obsidian 구문 규칙 | 일화(링크 파손 보고) | 아님 (내용) |
| 본문은 파일 편집, 이름변경·이동·속성은 Obsidian CLI(직렬, 출력 파싱) | 벤더 + 일화 | 아님 |
| Bases + `base:query`로 구조 질의, Dataview·MCP 미도입 | 일화·2차 | 아님 |
| `.obsidian/` 상태 무시, LF 강제, pull 먼저, 동기화 도구 하나만 | 일화·합의 | git 제공 |
| append-only 로그에 `merge=union` (미검증) | 추론 | git 제공 |
| 기기 간 기억은 vault에, 네이티브 메모리는 캐시로 | 벤더(기기 로컬 명시) | 제공 (메모리), 위키는 아님 |

## 채택하지 말 것

**공유 Neo4j Community(또는 Memgraph) 서버.** 저장소의 9월 22일 계획안과 ADR 초안이 권하지만 "상시 서버 없음" 제약과 직접 충돌하고, 그 논거인 다기기·다Host 동시 접속은 git 동기화와 기기별 결정론적 재구축으로 해결된다. 두 제품은 서버 전용이며(Neo4j GPLv3, Memgraph BSL 1.1), 코퍼스가 0에 가까운 첫 해에는 재구축 비용이 사실상 0이다. 그래프가 필요해지는 날에도 첫 선택은 임베디드 파생 인덱스다.

**LLM 추출 엔티티 그래프와 커뮤니티 요약(GraphRAG, LightRAG).** 독립 재시험에서 생성 텍스트가 검색 단위가 되면 단일 홉 QA가 붕괴하고(LightRAG NQ 16.6 F1 대 밀집 61.9), 인덱싱은 40~60배 비싸며, 추출 모델과 온도에 따라 결과가 달라져 증분 재구축의 결정론을 깨뜨린다. 관계는 저자가 쓴다.

**임베딩 인덱스 선도입.** 소코퍼스에서 grep·read 루프가 가장 정확하다는 측정이 있고, Cursor의 +12.5%는 1,000개 이상 파일의 코드베이스에 코드 임베딩을 쓴 벤더 수치라 수백 페이지 노트에 전이되지 않는다. 미스 로그가 승격 근거다.

**수치형 신뢰도와 시간 감쇠.** "거짓 정밀도"라는 운영자 반박이 있고, 교훈 저장소에서 "오래됨"은 "낡음"이 아니다(6개월 전 버그가 재발을 막는다). 명시적 supersede가 지원되는 메커니즘이다.

**"KB를 신뢰하라"는 에이전트 지시.** stale-document 연구에서 문서 추종 지시는 오답 반전을 30~37%에서 66~75%로 두 배 키웠다. 대신 `status`·`stale_after`·`superseded_by` 확인을 지시한다.

**실행 에이전트의 위키 직접 열람과 위키 전체 주입.** WikiSkill 절제(63.7%에서 60.9%)와 두 하네스의 컨텍스트 예산(200줄, 1,536자, 2%) 모두가 반대한다. 계획이 컴파일하고 실행자는 저장소만 grep한다.

**세션 JSONL 직접 파싱.** Anthropic이 형식을 "내부용이며 버전 간 변경"으로 명시했다. `SessionEnd` 훅과 `claude -p --resume`가 지원 경로다.

**주제별 하위 인덱스와 메타 라우터.** 측정된 실패다(계층 라우팅이 En.MC 0.9126을 0.6398로 떨어뜨림). 카탈로그는 한 단계 평면이다.

**벤더 벤치마크를 선택 기준으로.** Mem0·Zep이 같은 벤치마크를 65.99%, 75.14%, 58.44%로 다투고, LLM 판정 승률은 제시 순서로 뒤집힌다. 선택 기준은 autodev 자신의 미스 로그와 CI 데이터다.

**두 동기화 시스템 겹치기, 바이너리 인덱스 git 커밋, Dataview·MCP 초기 도입.** 각각 손상 경고, 단일 작성자 잠금 충돌, 이중 캐시 드리프트와 상시 프로세스 요구가 근거다.

## 결론

이 조사에서 가장 뜻밖의 결과는 측정 근거가 가장 강한 권고들이 가장 값싼 권고들이라는 점이다. 자기완결적 절과 3인칭 트리거 문장, 평면 카탈로그, 검색 전 질의 재작성, 유효 경계 문장이 붙은 supersede, 실행자에게 위키를 주지 않는 역할 분리는 모두 파일과 지침 한 단락으로 끝나는데, 각각 청킹 연구, MCP 설명 연구, progressive disclosure 연구, stale-document 연구, WikiSkill 절제라는 통제 실험을 등에 지고 있다. 반대로 비용이 큰 선택(그래프 DB, 임베딩 인덱스, 시간 그래프 메모리)은 벤더 수치나 휴리스틱에 기대고 있고, 독립 재시험은 그 이득을 몇 점으로 줄이거나 회귀로 뒤집었다. 1인 개발자에게 이 비대칭은 설계 원칙이 된다. 인프라가 아니라 규약에 투자하고, 인프라는 자기 로그가 요구할 때 산다.

두 번째 함의는 측정의 자리다. 첫 해의 코퍼스는 어떤 문헌의 임계값도 넘지 않으므로, 승격 결정은 외부 벤치마크가 아니라 autodev가 스스로 남기는 세 가지 기록(못 찾은 질의, 다중 홉 질의의 비율, 스킬 적용 뒤 같은 실패 서명의 재발)에서 나온다. 이 기록은 4절의 inbox 노트와 같은 형식이므로 지식 루프가 자기 승격 조건을 스스로 생산한다. 남은 공백도 분명하다. 한영 혼용 개인 Markdown 코퍼스에 대한 에이전트 검색 벤치마크, 저자 작성 관계 대 LLM 추출 그래프의 검색 품질 비교, 사람 게이트 대 자동 게이트의 통제 비교는 2026년 9월 기준 어디에도 없다. 그 공백 위에서 지금 할 수 있는 가장 정직한 일은 저장소에 남은 공유 Neo4j 계획안을 파생 인덱스 방향과 정합시키고, 미스 로그를 첫날부터 켜는 것이다.
