# M32 0.0.3-T004 — First-frame Capture Host

Status: IMPLEMENTATION BASELINE
Version: `0.0.3 First Frame`
Task: `0.0.3-T004`

## Purpose

T003 proved a synthetic M32 MIDlet can reach `MIDlet.startApp()` and Running state.

T004 establishes the deterministic host-side observation point that later tasks will use to decide
whether the guest has produced its first actual frame.

T004 deliberately does not modify the guest MIDlet to create LCDUI objects yet.

## First-frame Definition

For First Frame integration tests, the first frame is:

```text
the first valid M32 RgbaFrame delivered to DisplayHost::present_rgba8
```

A resize or redraw request is **not** a frame.

This distinction prevents false positives where the guest initializes a screen size but never paints
content.

## Capture Host

Test-only:

```text
FirstFrameCaptureDisplayHost
```

State:

```text
DisplaySize
redraw_count
present_count
first_frame: Option<RgbaFrame>
```

Behavior:

```text
resize
-> records display size

request_redraw
-> increments redraw count
-> does not create a frame

first present_rgba8
-> increments present count
-> stores frame permanently as first_frame

later present_rgba8
-> increments present count
-> does not replace first_frame
```

The immutable first-frame rule is important. Later animation or repaint activity must never make a
first-frame test nondeterministic.

## WIE Bridge Integration

T004 verifies:

```text
Synthetic WIE Image
-> WieScreenAdapter::paint
-> canonical M32 RGBA8 conversion
-> FirstFrameCaptureDisplayHost
```

Expected synthetic pixels:

```text
[10, 20, 30, 255]
[40, 50, 60, 128]
```

## Host Injection

The existing test platform builder is extended with:

```text
recording_platform_hosts_with_display_and_observers(...)
```

This allows T005/T006 to attach the same capture host to a real J2ME session while retaining stdout
and filesystem observers.

## Tests

```text
first_frame_capture_host_does_not_invent_frame_before_present
first_frame_capture_host_locks_the_first_presented_frame
wie_screen_adapter_feeds_rgba8_into_first_frame_capture_host
```

## Non-goals

T004 does not:

- change public M32 emulator API;
- change production desktop rendering;
- modify the Java guest fixture;
- construct `Display` or `Canvas`;
- execute `Canvas.paint`;
- tick until a guest-generated frame.

Those begin with T005 and T006.

## Dependencies

No new dependency.

No Cargo.lock change.

No WIE/RustJava revision change.

The T003 RustJava local compatibility patch remains required and unchanged.

## Acceptance

- resize/redraw cannot fake first-frame completion;
- first valid present is captured;
- later presents cannot overwrite first frame;
- WIE Screen adapter feeds canonical RGBA8 into capture host;
- capture display can be injected into the complete test platform;
- previous T003 verifier remains green;
- all quality gates pass.
