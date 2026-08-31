# M32 0.0.4-T010 — Input Canonical Integration / Version Close

Status: IMPLEMENTATION BASELINE
Version: `0.0.4 Input`
Task: `0.0.4-T010`

## Purpose

Close the complete `0.0.4 Input` version.

T010 adds no new emulator runtime behavior.

It proves that the contracts from T001 through T009 coexist as one reproducible input baseline and
that the repository is ready to move to `0.0.5 Audio`.

## Closed Task Set

```text
T001 Stable M32Key / GuestInputEvent Contract
T002 EmulatorSession Input Dispatch Seam
T003 Exact M32 -> pinned WIE Key/Event Mapping
T004 Deterministic Key-observer MIDlet Fixture
T005 Real KeyDown -> Canvas.keyPressed Proof
T006 Real KeyUp/KeyRepeat Callback Proof
T007 Full 24-key MIDP Code Matrix End-to-End
T008 Deterministic 350ms / 12Hz Key Repeat Policy
T009 Maximum Six Held Guest Keys
T010 Input Canonical Integration / Version Close
```

## Evidence Gate

The version-close verifier requires the established M32 evidence filenames:

```text
M32_0.0.4-T001_evidence.md
M32_0.0.4-T002_evidence.md
M32_0.0.4-T003_evidence.md
M32_0.0.4-T004_evidence.md
M32_0.0.4-T005_evidence.md
M32_0.0.4-T006_evidence.md
M32_0.0.4-T007_evidence.md
M32_0.0.4-T008_evidence.md
M32_0.0.4-T009_evidence.md
```

## Canonical Input Contract

The complete version closes:

```text
24 stable feature-phone keys
KeyDown / KeyUp / KeyRepeat phases
backend-independent EmulatorSession input seam
exact M32 -> pinned WIE key mapping
real WIE EventQueue key dispatch
real Display -> Canvas virtual key callback
exact 24-key MIDP integer matrix
350ms initial repeat delay
12Hz repeat frequency
origin-based non-drifting repeat calculation
release stops repeats
maximum six simultaneously held guest keys
duplicate key-down suppression
deterministic multi-key repeat ordering
```

## Real Guest Proof

The close verifier invokes Bundle B, which proves the real Java path:

```text
M32Key
-> GuestInputEvent
-> WieSession::handle_input
-> WIE backend Event
-> MIDP EventQueue
-> Display.handleKeyEvent
-> Canvas.handleKeyEvent
-> KeyCanvas.keyPressed / keyReleased / keyRepeated
```

## Policy Proof

The close verifier invokes Bundle C functional verification, which proves:

```text
350ms repeat start
12Hz repeat cadence
catch-up without cumulative schedule drift
release stops repeat
six-held-key ceiling
seventh-key rejection
duplicate-key suppression
capacity reuse after release
stable repeat ordering
```

## Workspace-local Dependency Boundary

The only new production dependency edge introduced by Bundle C is:

```text
m32-input -> m32-emulator-api
```

The close verifier inspects Cargo metadata and requires:

```text
m32-input has exactly one normal dependency
that dependency is m32-emulator-api
the dependency is workspace-local/path-based
```

No external package is allowed to enter `m32-input`.

The intentional `Cargo.lock` update caused by this workspace-local dependency edge is allowed.

## Quality Gates

The close verifier runs:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check --workspace --all-targets
git diff --check
```

## Regression Baseline

Expected minimum:

```text
m32-desktop          9
m32-domain           3
m32-emulator-api    32
m32-input             6
m32-test-fixtures     2
m32-wie-adapter      66
-----------------------
minimum total       118
```

## Vendor Boundary

No new WIE or RustJava source modification is part of `0.0.4`.

The close verifier rejects an uncommitted change to:

```text
third_party/rustjava/jvm/src/jvm.rs
```

The existing T003 RustJava compatibility patch remains committed history.

## Out of Scope

`0.0.4` does not claim:

```text
finished desktop keyboard preferences UI
controller remapping UX
Auto Keypad
per-title key profiles
audio completion
storage/save completion
broad real-game compatibility
```

The immediate next version is:

```text
0.0.5 Audio
```

## Acceptance

After the close verifier returns exit code `0` and repository scope contains only intended Bundle C
files/evidence/T010 close files:

```text
0.0.4 Input = 10 / 10 DONE
```
