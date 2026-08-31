# M32 0.0.3-T008 — First-frame Timeout/Failure Boundary

Status: IMPLEMENTATION BASELINE
Version: `0.0.3 First Frame`
Task: `0.0.3-T008`

## Purpose

T006/T007 prove the successful first-frame path.

T008 defines the negative boundary.

The harness must distinguish three outcomes:

```text
1. frame captured
2. backend/session fault before frame
3. bounded timeout with a healthy session but no frame
```

A backend fault must never be mislabeled as a timeout.

A healthy MIDlet that simply never paints must never be mislabeled as a backend fault.

## Test-only Result Boundary

The T006 harness is split into:

```text
try_tick_until_first_captured_frame(...)
tick_until_first_captured_frame(...)
```

The lower-level function returns:

```text
Ok(RgbaFrame)

Err(SessionFault {
    tick,
    EmulatorSessionError
})

Err(Timeout {
    max_ticks,
    redraws,
    presents,
    SessionState
})
```

The existing T006/T007 convenience wrapper preserves its panic-on-test-failure behavior.

No public emulator API changes.

## Timeout Fixture

T008 reuses the already-locked T003:

```text
m32.RunningMidlet
```

That guest:

```text
loads successfully
executes startApp()
prints M32_FIRST_FRAME_BOOT_OK
reaches SessionState::Running
never constructs Display/Canvas
never requests redraw
never presents a frame
```

The boundary test reuses the already-locked First Frame boot bound:

```text
512 ticks
```

Expected:

```text
Timeout
max_ticks = 512
redraws = 0
presents = 0
state = Running
first_frame = None
stdout contains M32_FIRST_FRAME_BOOT_OK
```

This proves timeout means:

```text
healthy guest, but no first frame within the bounded wait
```

rather than boot failure.

## Backend Fault Fixture

T008 reuses the Core Adapter missing-main JAR.

Expected:

```text
SessionFault
error.code = BackendTickFailed
state = Faulted
first_frame = None
present_count = 0
```

The fault must be returned before the timeout boundary.

This proves:

```text
backend failure != first-frame timeout
```

## Existing Success Path

The successful PaintMidlet path remains unchanged:

```text
Ok(RgbaFrame)
```

and all T007 exact-content tests are rerun through the verifier chain.

## Tests

```text
first_frame_wait_times_out_cleanly_when_running_midlet_never_paints
first_frame_wait_reports_backend_fault_before_timeout
```

## Non-goals

T008 does not introduce a production user-facing timeout error code.

This task locks the integration-test boundary only.

Product/runtime error UX belongs to later session/application integration work.

## Dependencies

No new dependency.

No Cargo.lock change.

No WIE/RustJava change.

No guest fixture change.

## Expected Test Count

T007:

```text
56 adapter tests
```

T008 adds two:

```text
58 adapter tests
```

Minimum workspace unit tests:

```text
101
```
