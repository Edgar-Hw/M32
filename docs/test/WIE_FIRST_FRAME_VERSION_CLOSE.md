# M32 0.0.3-T010 — First Frame Version Close / Gates

Status: IMPLEMENTATION BASELINE
Version: `0.0.3 First Frame`
Task: `0.0.3-T010`

## Purpose

T010 closes the complete `0.0.3 First Frame` version.

No new emulator behavior is introduced.

The task verifies that the entire First Frame version is internally consistent, reproducible, clean,
and ready to hand off to `0.0.4 Input`.

## Version Scope

The version contains exactly ten logical tasks:

```text
T001 Explicit JAD+JAR launch path
T002 Deterministic bootable MIDlet fixture
T003 Positive boot-to-Running smoke
T004 First-frame capture host
T005 Deterministic Canvas/Paint fixture
T006 Tick-until-first-frame harness
T007 RGBA8 frame dimension/content lock
T008 First-frame timeout/failure boundary
T009 First-frame integration verifier
T010 First Frame version close/gates
```

T010 requires T001 through T009 to be committed before it is applied.

## Version-close Evidence Gate

The repository must contain the established M32 task-evidence naming convention:

```text
docs/spec/task-evidence/M32_0.0.3-T001_evidence.md
docs/spec/task-evidence/M32_0.0.3-T002_evidence.md
docs/spec/task-evidence/M32_0.0.3-T003_evidence.md
docs/spec/task-evidence/M32_0.0.3-T004_evidence.md
docs/spec/task-evidence/M32_0.0.3-T005_evidence.md
docs/spec/task-evidence/M32_0.0.3-T006_evidence.md
docs/spec/task-evidence/M32_0.0.3-T007_evidence.md
docs/spec/task-evidence/M32_0.0.3-T008_evidence.md
docs/spec/task-evidence/M32_0.0.3-T009_evidence.md
```

Missing task evidence blocks the version close.


### Evidence filename convention

T010 follows the repository convention already used by earlier task evidence:

```text
M32_<TaskId>_evidence.md
```

For example:

```text
M32_0.0.3-T001_evidence.md
```

The close verifier must not require a second renamed copy such as `0.0.3-T001.md`.

## Canonical Integration Gate

T010 invokes:

```text
scripts/verify-wie-first-frame-integration.ps1
```

This proves the full T001-T009 First Frame chain and the canonical:

```text
SUCCESS
TIMEOUT
FAULT
```

contract.

## Required Quality Gates

The close verifier runs:

```text
cargo fmt --all -- --check

cargo clippy --workspace --all-targets -- -D warnings

cargo test --workspace

cargo check --workspace --all-targets

git diff --check
```

All must return exit code `0`.

## Regression Baseline

Expected minimum M32 unit tests:

```text
m32-desktop          9
m32-domain           3
m32-emulator-api    29
m32-test-fixtures    2
m32-wie-adapter     58
----------------------
minimum total       101
```

T010 adds no unit test, so the count remains unchanged.

## Dependency / Vendor Working-tree Boundary

T010 rejects an uncommitted change to:

```text
Cargo.lock
third_party/rustjava/jvm/src/jvm.rs
```

The documented RustJava compatibility patch introduced by T003 is already committed history.

T010 requires no additional RustJava modification.

The WIE revision remains pinned and is revalidated indirectly through the T009 integration chain.

## First Frame Contract Closed by 0.0.3

On successful close, the following are established:

```text
explicit JAD+JAR launch
deterministic legal synthetic guest fixtures
real RustJava application class loading
MIDlet.startApp positive boot
Ready -> Running lifecycle
host first-frame capture semantics
real Display.setCurrent repaint request
deterministic executor-time progression
MIDP EventQueue repaint dispatch
real Canvas.paint(Graphics)
WIE screen.paint integration
canonical M32 RGBA8 conversion
actual 176x220 first frame
exact 154880-byte framebuffer
exact 38720-pixel framebuffer
exact 16x16 M32 RED region
exact BG0 remainder
healthy Running no-frame timeout
backend fault precedence over timeout
canonical SUCCESS/TIMEOUT/FAULT integration verifier
```

## Out of Scope

`0.0.3` does not claim:

```text
keyboard/keypad input
interactive control
production desktop game renderer
audio playback completion
persistent game save support
real-game compatibility breadth
```

Those belong to later versions.

The immediate next version is:

```text
0.0.4 Input
```

## Acceptance

T010 is DONE only when:

```text
all T001-T009 evidence exists
canonical T009 verifier passes
rustfmt passes
Clippy -D warnings passes
workspace tests pass
workspace all-target check passes
git diff --check passes
Cargo.lock has no uncommitted change
RustJava jvm compatibility source has no uncommitted change
repository scope contains only intended T010 close files
```

After T010 commit and CI green:

```text
0.0.3 First Frame = 10 / 10 DONE
```
