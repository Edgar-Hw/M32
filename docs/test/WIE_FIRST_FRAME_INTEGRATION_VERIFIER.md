# M32 0.0.3-T009 — First-frame Integration Verifier

Status: IMPLEMENTATION BASELINE
Version: `0.0.3 First Frame`
Task: `0.0.3-T009`

## Purpose

T001 through T008 established the First Frame path in separate layers.

T009 creates one canonical integration verifier that proves the complete contract without adding a
new runtime feature.

The verifier covers:

```text
launch descriptor
deterministic guest fixtures
positive MIDlet boot
RustJava application class loading compatibility
capture-host semantics
real Canvas repaint path
exact 176x220 RGBA8 content
healthy-no-frame timeout
backend-fault precedence
adapter/API regression suites
```

T010 remains responsible for the final version-close quality gates and repository release boundary.

## Canonical Three-outcome Contract

The First Frame wait has exactly three integration outcomes:

```text
SUCCESS
Ok(RgbaFrame)

TIMEOUT
healthy Running session
no frame within the bounded wait

FAULT
backend/session failure before the frame
```

The canonical verifier requires all three branches to remain valid.

## Locked Fixtures

### Positive boot / healthy no-frame

```text
j2me-first-frame-running.jad
SHA-256 e02fba5850f154913d1ed45d845c63f4cad53a2f1d66d348f5046a83c56a1ae7

j2me-first-frame-running.jar
SHA-256 2521a16329f92ec6eaf07a2d9bc379713261c00146a722c14b5dd0aab5bb465d
```

### Paint success

```text
j2me-first-frame-paint.jad
SHA-256 7fe94a7cb014c40367ee2139d25cf4e1c3cde37d8e9d73bcb1277a4a1f9efc33

j2me-first-frame-paint.jar
SHA-256 717063ece69306b9338da722d57f14815987c6a044f738da3b2e95373a9f8b5a
```

The backend-fault branch reuses the already-locked Core Adapter missing-main fixture.

## Static Contract Markers

The verifier checks that adapter source still contains:

```text
FirstFrameCaptureDisplayHost
DeterministicAdvancingClock
FirstFrameWaitError
SessionFault
Timeout
first_frame_paint_fixture_ticks_until_guest_frame_is_captured
first_frame_paint_fixture_locks_exact_rgba8_dimensions_and_content
first_frame_wait_times_out_cleanly_when_running_midlet_never_paints
first_frame_wait_reports_backend_fault_before_timeout
```

These are integration-test markers, not public API.

## Dynamic Verification

The canonical verifier:

```text
1. validates locked fixture hashes
2. validates required source markers
3. runs the full T008 verifier chain
4. runs the complete m32-wie-adapter suite
5. runs the complete m32-emulator-api suite
```

Expected adapter count:

```text
58
```

Expected API count:

```text
29
```

No new unit test is introduced by T009.

## Non-goals

T009 does not:

- change runtime emulation behavior;
- add new First Frame tests;
- alter fixtures;
- add dependencies;
- change Cargo.lock;
- replace T010 version-close gates.

## Acceptance

The verifier must finish with all canonical First Frame branches and the previous T001-T008 chain
green.

Final workspace/clippy/check/repository-close validation belongs to T010.
