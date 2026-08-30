# M32 Panic / Crash Report Policy

Status: LOCKED BASELINE
Task: `0.0.1-T010`

## 1. 목적

M32는 process panic이 발생했을 때 Rust 기본 panic 문자열에만 의존하지 않고,
BuildInfo와 최소 진단 필드를 포함한 M32 crash report를 출력한다.

T010의 crash report는 **stderr fallback report**다.

영구 crash file 저장 경로는 `0.0.1-T011`에서 `%LOCALAPPDATA%\M32` data directory가
고정된 뒤 연결한다. T010은 임의의 current directory/temp directory를 영구 crash
저장소로 사용하지 않는다.

## 2. Crash Report Schema

초기 schema:

```text
schema_version=1
```

필드:

- app version
- Product Spec version
- Spec Bundle version
- Git commit
- WIE baseline commit
- Rust version
- target
- build profile
- thread name
- sanitized panic message
- source file
- source line
- source column

wall-clock timestamp와 backtrace는 T010 baseline에 포함하지 않는다.

## 3. Panic Message Safety

panic message는:

- 최대 2048 characters
- CR/LF/TAB을 space로 치환
- 기타 control character를 replacement character로 치환

한다.

Crash report는 자동 업로드되지 않는다.

panic message는 향후 사용자 입력이 포함될 가능성이 있으므로 release diagnostic bundle에
포함할 때는 별도 redaction/export review가 필요하다.

## 4. Source Path Safety

Rust panic location이 absolute path라면 source file basename만 report한다.

relative path라면 `/` separator로 정규화하여 유지한다.

이는 developer machine의 absolute source checkout path가 crash output에 남는 것을 줄인다.

## 5. Backtrace

T010에서는 backtrace를 기본 수집하지 않는다.

이유:

- host absolute path 노출 가능성
- 매우 큰 diagnostic payload
- symbol 환경에 따른 비결정적 결과

backtrace가 필요해질 경우 별도 diagnostic policy와 함께 추가한다.

## 6. Structured Crash Event

panic hook은 가능한 경우 `m32::crash` target으로 다음 safe field를 기록한다.

- `event="panic"`
- crash report schema version
- app version
- git commit
- thread name

전체 panic message는 structured INFO/ERROR event field로 복제하지 않는다.
상세 fallback report는 stderr에만 출력한다.

## 7. Intentional Smoke Test

Debug build에서만 다음 환경 변수를 acceptance test 용도로 사용한다.

```text
M32_CRASH_TEST=panic
```

설정 시 normal startup event 이후 의도적인 panic을 발생시킨다.

Release build에서는 이 smoke-test trigger로 panic하지 않는다.

이 기능은 실제 사용자 기능이 아니라 Foundation crash-hook 검증용이다.

## 8. Exit Behavior

일반 Rust unwind panic exit behavior를 유지한다.

Windows debug `cargo run` smoke test에서는 일반적으로 process exit code `101`이 예상된다.

T010은 panic 후 자동 restart, recovery, save-state 복구를 구현하지 않는다.
