# M32 Foundation Baseline Benchmark

Status: LOCKED BASELINE
Task: `0.0.1-T012`
Schema: `1`

## 1. 목적

Foundation 단계가 끝나는 시점의 **재현 가능한 synthetic performance workload**를 저장소에
고정한다.

이 benchmark는 M32 emulator 성능 목표나 release pass/fail threshold가 아니다.

T012의 역할은:

- 동일 workload를 모든 세션에서 반복할 수 있게 만들기;
- toolchain/profile 변경 후 큰 performance regression을 비교할 기준 마련;
- 240×320 feature-phone class framebuffer 작업에 가까운 최소 CPU/memory workload 제공;
- 향후 실제 WIE/display benchmark가 생기기 전까지 Foundation baseline 보존;

이다.

## 2. Cargo Bench Profile

```toml
[profile.bench]
inherits = "release"
debug = 1
lto = "thin"
codegen-units = 1
incremental = false
```

`target-cpu=native`는 사용하지 않는다.

이유는 CPU별 binary 특성을 baseline 계약에 섞지 않고 Windows x64 MSVC target 기준을
유지하기 위해서다.

## 3. Workloads

Schema v1은 정확히 세 workload를 가진다.

### `rgba_copy_240x320`

- logical framebuffer: 240×320
- RGBA8
- iterations: 4000
- 목적: frame-sized memory copy baseline

### `rgb565_to_rgba_240x320`

- logical framebuffer: 240×320
- RGB565 input → RGBA8 output
- iterations: 1000
- 목적: feature-phone style pixel conversion baseline

### `integer_scale_3x_240x320`

- logical framebuffer: 240×320
- nearest integer 3× → 720×960
- iterations: 80
- 목적: M32 Pixel Perfect 계열 작업과 유사한 CPU-side reference workload

이 코드는 실제 `m32-display` 구현이 아니다.
Display Engine Task에서 이 benchmark 코드를 production renderer로 재사용하지 않는다.

## 4. 출력 계약

각 run은 다음 header를 출력한다.

```text
M32_BASELINE_BENCHMARK schema_version=1
logical_width=240
logical_height=320
```

각 workload는:

```text
M32_BENCH name=<id> iterations=<n> total_ns=<n> ns_per_iteration=<n> checksum=<n>
```

형식을 사용한다.

마지막:

```text
M32_BASELINE_BENCHMARK result=PASS
```

가 있어야 한다.

## 5. Acceptance와 Performance Gate의 차이

T012 acceptance:

- benchmark target이 stable Rust 1.98.0에서 compile/run 된다;
- 세 workload가 모두 실행된다;
- elapsed time이 0보다 크다;
- checksum이 non-zero다;
- 출력 schema가 유지된다.

**절대 시간 threshold는 T012 acceptance에 사용하지 않는다.**

노트북 전원 상태, CPU, background process, thermal state 때문에 단일 wall-clock 수치를
CI hard gate로 쓰면 false failure가 발생하기 때문이다.

성능 회귀 판정은 향후 dedicated benchmark/release performance Task에서 machine/profile
조건과 반복 통계를 고정한 뒤 도입한다.

## 6. Canonical Command

```powershell
powershell -ExecutionPolicy Bypass -File scripts\run-foundation-benchmark.ps1
```

또는:

```powershell
cargo bench -p m32-test-fixtures --bench foundation_baseline
```

## 7. Baseline Evidence

T012 완료 evidence에는 최소 다음을 기록한다.

- rustc version
- Cargo version
- benchmark schema
- 세 workload의 실제 출력
- benchmark process exit code
- quality gate 결과

Machine-specific benchmark 숫자를 LOCK 값으로 취급하지 않는다.

## 8. 변경 관리

다음 변경은 기존 baseline과 직접 비교가 깨질 수 있으므로 새 schema 또는 RFC/Task 판단이
필요하다.

- workload algorithm 변경
- framebuffer dimensions 변경
- iteration count 변경
- benchmark profile 변경
- output field 의미 변경

단순 compiler/runtime 성능 개선으로 측정값만 달라지는 것은 schema 변경이 아니다.
