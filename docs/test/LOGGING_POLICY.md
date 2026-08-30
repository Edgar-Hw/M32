# M32 Logging Policy

Status: LOCKED BASELINE
Task: `0.0.1-T009`

## 1. 목적

M32는 `tracing` 기반의 구조화된 application diagnostics를 사용한다.

T009의 범위는 **stderr 기반 structured logging 초기화**까지다.

파일 로그 저장 위치와 rotation은 `0.0.1-T011`의 M32 data-directory 구조가 고정된
이후 연결한다. T009에서 임의의 현재 디렉터리나 임시 경로에 영구 로그를 만들지 않는다.

## 2. 기본 Level

기본 level:

```text
INFO
```

환경 변수:

```text
M32_LOG
```

허용 값:

```text
trace
debug
info
warn
error
```

대소문자를 구분하지 않는다.

잘못된 값이면 INFO로 fallback하고 stderr에 짧은 경고를 출력한다.

## 3. Level 의미

- `ERROR`: 요청/작업을 완료할 수 없거나 데이터 손상 위험이 있는 실패
- `WARN`: fallback, degraded mode, 복구 가능한 비정상 상태
- `INFO`: app/game lifecycle과 중요한 상태 전환
- `DEBUG`: 개발·진단 목적의 상세 상태
- `TRACE`: 높은 빈도의 세밀한 흐름. 기본 비활성

## 4. Startup Event

M32 startup은 `m32::lifecycle` target의 `app_start` event로 기록한다.

필드:

- app version
- Product Spec version
- Spec Bundle version
- Git commit
- WIE baseline commit
- Rust version
- build target
- build profile

BuildInfo의 값을 복제해서 새 문자열을 따로 관리하지 않는다.

## 5. Privacy / Safety

기본 INFO 로그에 다음을 기록하지 않는다.

- game binary bytes
- save bytes
- extracted copyrighted resource bytes
- user document contents
- arbitrary imported text contents

사용자 로컬 파일의 전체 absolute path는 INFO level에서 기본 기록하지 않는다.
경로가 진단에 필요할 경우 후속 기능에서 명시적 redaction/diagnostic export 규칙을 적용한다.

M32 T009 logging은 network telemetry가 아니다.

- 로그 자동 업로드 없음
- analytics 없음
- 계정 없음
- 원격 collector 없음

## 6. Formatting

T009 console logger는:

- compact human-readable format
- target 표시
- thread ID 표시
- thread name 표시
- ANSI color 비활성

을 사용한다.

ANSI를 끄는 이유는 GitHub Actions/PowerShell/diagnostic copy-paste에서 escape sequence가
섞이지 않게 하기 위해서다.

## 7. Dependency Baseline

T009 direct dependencies:

- `tracing = 0.1.44`
- `tracing-subscriber = 0.3.23`

Cargo workspace에서는 exact version requirement를 사용한다.

`tracing-subscriber`는 default features를 끄고 `fmt`, `std`만 활성화한다.
T009에서는 EnvFilter/JSON/ANSI 기능을 도입하지 않는다.
