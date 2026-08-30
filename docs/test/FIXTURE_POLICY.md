# M32 Synthetic Fixture Policy

Status: LOCKED BASELINE
Task: `0.0.1-T007`

## 1. 목적

M32 저장소와 CI는 상용 피처폰 게임을 테스트 fixture로 포함하지 않는다.

테스트는 가능한 한 다음 순서의 자료만 사용한다.

1. M32가 직접 작성/생성한 synthetic fixture
2. 재배포 권한과 라이선스가 확인된 upstream fixture
3. parser/security 검증을 위해 직접 만든 최소 malformed fixture

개발자가 개인적으로 보유한 상용 게임 파일은 로컬 호환성 시험에 사용할 수 있지만
Git 추적, GitHub Actions artifact, release artifact에는 포함하지 않는다.

## 2. Fixture Manifest 의무 필드

`assets/fixtures/fixture-manifest.json`의 모든 fixture는 최소 다음을 가진다.

- `id`: 저장소 전체에서 고유한 kebab-case 식별자
- `kind`: fixture 종류
- `path`: 저장소 root 기준 상대 경로
- `license`: 재배포 근거
- `source`: 생성/출처 설명
- `sha256`: 실제 파일의 lowercase SHA-256
- `purpose`: 이 fixture가 검증하는 동작
- `redistribution_status`: `redistributable`만 repository tracked fixture로 허용

fixture byte가 하나라도 바뀌면 manifest SHA-256도 같은 commit에서 갱신해야 한다.

## 3. 금지 자료

명시적 재배포 권한 없이 다음을 fixture로 commit하지 않는다.

- 상용 JAR/JAD
- WIPI/SKVM/Clet/LGT 게임 binary
- 실제 게임에서 추출한 image/audio/font/text
- 사용자 save 데이터
- ROM/firmware dump
- 실제 게임 파일의 부분 byte를 잘라 만든 fixture

malformed fixture도 상용 파일을 손상시켜 만드는 것이 아니라 M32가 처음부터 직접 생성한다.

## 4. J2ME Source Fixtures

T007에는 향후 J2ME test package를 만들기 위한 self-authored source fixture를 포함한다.

- `j2me-hello-source`
- `j2me-input-source`
- `j2me-audio-source`

이 파일은 M32가 직접 작성한 테스트 소스이며, 아직 T007에서 runnable JAR를 생성하거나
J2ME runtime dependency를 추가하지 않는다.

해당 source가 향후 compiled JAR fixture로 승격될 경우 새 artifact 자체의 SHA-256과
빌드 재현 방법을 manifest에 별도 항목으로 추가한다.

## 5. Malformed Fixtures

T007에는 parser/security 테스트를 위한 아주 작은 malformed file을 포함한다.

- truncated ZIP signature
- truncated Java CLASS header
- truncated PNG signature

이 파일들은 의도적으로 유효하지 않다. 정상 asset으로 해석하거나 실행하면 안 된다.

## 6. 검증

Canonical 명령:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\verify-fixtures.ps1
```

검증기는 다음을 확인한다.

- manifest JSON parse
- fixture ID 중복 없음
- 필수 필드 존재
- 모든 path가 `assets/fixtures/` 아래에 존재
- SHA-256 일치
- `redistribution_status = redistributable`
- path traversal/absolute path 금지

## 7. 변경 규칙

fixture 추가/수정 commit은 반드시 다음을 함께 포함한다.

1. fixture file
2. manifest entry
3. license/source/purpose
4. new SHA-256
5. fixture verifier PASS evidence

상용 게임으로만 재현되는 버그는 상용 파일을 commit하는 대신
재현 가능한 최소 synthetic case를 먼저 만들도록 시도한다.
