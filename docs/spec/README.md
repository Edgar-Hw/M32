# M32 Specification Directory

이 디렉터리는 M32의 제품·구현·변경 관리 문서를 저장한다.

## 문서 권위

M32의 권위 순서는 다음과 같다.

1. MASTER SPEC
2. LOCKFILE
3. IMPLEMENTATION SPEC
4. DESIGN SYSTEM
5. TEST SPEC
6. TASK REGISTRY
7. SESSION PROTOCOL
8. WORKLOG

저장소 내부의 ADR/RFC/작업 evidence는 위 문서를 보조하며, 승인되지 않은 메모가
MASTER/LOCKFILE을 자동으로 덮어쓰지 않는다.

## 디렉터리

```text
docs/
├─ adr/
│  ├─ ADR_TEMPLATE.md
│  └─ ADR-NNNN-short-title.md
├─ rfc/
│  ├─ RFC_TEMPLATE.md
│  └─ RFC-NNNN-short-title.md
└─ spec/
   ├─ README.md
   └─ task-evidence/
      └─ <TaskID>.md
```

## ADR 사용 시점

ADR은 **결정된 기술/제품 선택**을 기록한다.

예:

- Windows-first 채택 이유
- SQLite 채택 이유
- 특정 렌더링 구조 선택
- 외부 dependency를 adapter 뒤에 두는 결정

파일명:

```text
ADR-NNNN-short-title.md
```

번호는 4자리 zero-padding을 사용한다.

예:

```text
ADR-0001-windows-first.md
```

## RFC 사용 시점

RFC는 **현재 고정 규칙을 바꾸거나 큰 새 범위를 제안할 때** 사용한다.

다음은 RFC 없이 변경하면 안 된다.

- MASTER SPEC의 LOCK 항목
- Task 순서/삭제/병합
- 공개 stable ID/error code/config key
- DB 호환성
- 게임 binary identity
- 브랜드/카드 비율/navigation/core 디자인 방향
- v1.0 비목표를 v1.0 범위로 당기는 변경

파일명:

```text
RFC-NNNN-short-title.md
```

예:

```text
RFC-0001-change-ui-framework.md
```

## Task Evidence

모든 구현 Task는 완료 시 다음 파일을 남긴다.

```text
docs/spec/task-evidence/<TaskID>.md
```

Evidence에는 최소한 다음이 있어야 한다.

- Task ID
- 변경 파일
- 실제 검증 명령
- 실제 결과
- known failure
- 다음 정확한 Task ID

## 링크

- [ADR Template](../adr/ADR_TEMPLATE.md)
- [RFC Template](../rfc/RFC_TEMPLATE.md)
- [Game File Policy](../legal/GAME_FILE_POLICY.md)

## 변경 원칙

좋아 보인다는 이유만으로 기준서를 바꾸지 않는다.

필요한 변경은:

```text
문제 발견
→ Evidence 확보
→ RFC 작성
→ 영향 분석
→ 승인
→ ADR/Spec 갱신
→ 구현
→ 테스트
```

순서로 진행한다.
