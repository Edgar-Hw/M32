# M32 0.0.3-T006 — Tick-until-first-frame Harness

Status: IMPLEMENTATION BASELINE
Version: `0.0.3 First Frame`
Task: `0.0.3-T006`

## Purpose

T005 added a deterministic Canvas/Paint guest.

T006 proves that the guest can be ticked through the real pinned WIE/RustJava runtime until the M32
display host receives its first guest-generated `RgbaFrame`.

T006 is the first task that joins:

```text
real guest JAD/JAR
real Java class loading
MIDlet.startApp
Display.setCurrent
redraw scheduling
WIE event queue
Canvas.paint
WIE screen image
WieScreenAdapter
M32 FirstFrameCaptureDisplayHost
```

## Important redraw boundary

Pinned WIE separates redraw request from repaint event dispatch.

`Display.setCurrent()` calls:

```text
Display.repaint(...)
-> Platform.screen().request_redraw()
```

That is a frontend redraw request only.

The Java `Canvas.paint(Graphics)` path runs after the backend emulator receives:

```text
wie_backend::Event::Redraw
```

and the MIDP event loop dispatches that event as a repaint event.

Therefore a test that merely calls `tick()` and waits for a frame would be incomplete: the host-side
redraw acknowledgement must be pumped back into the emulator.

## Test-only frontend pump

T006 adds a test-only harness:

```text
tick_until_first_captured_frame(...)
```

For every tick:

```text
1. run the real M32 WieSession::tick()
2. observe FirstFrameCaptureDisplayHost.redraw_count
3. forward each unacknowledged redraw exactly once as wie_backend::Event::Redraw
4. inspect FirstFrameCaptureDisplayHost.first_frame
5. stop on the first captured frame
```

Maximum:

```text
512 ticks
```

The pump uses the existing private WIE emulator object inside `WieSession` only from the local
`#[cfg(test)]` module.

No public event/input API is introduced early.

The real Input Bridge remains the `0.0.4` version scope.

## Logical display size

Before J2ME construction the capture host is initialized to:

```text
176 x 220
```

Pinned WIE `Display::<init>` obtains the screen dimensions from the backend `Screen::width/height`
and builds its screen image using those dimensions.

Exact frame dimension/content assertions are intentionally deferred to T007.

T006 only requires a valid non-empty first frame.

## Required runtime proof

The T006 test requires:

```text
initial SessionState::Ready
initial first_frame = None
initial present_count = 0

real runtime tick succeeds
at least one redraw request occurs
redraw is forwarded to WIE EventQueue
a guest-generated RgbaFrame reaches the capture host
present_count >= 1
SessionState::Running
M32_FIRST_FRAME_CANVAS_READY stdout sentinel observed
```

## Test

```text
first_frame_paint_fixture_ticks_until_guest_frame_is_captured
```

## Non-goals

T006 intentionally does not lock:

- exact RGBA8 frame dimensions;
- exact corner pixels;
- exact BG0 pixel count;
- exact RED marker pixel count;
- full-frame hash.

Those are T007.

T006 also does not add public keyboard/input event handling.

## Dependencies

No new dependency.

No Cargo.lock change.

No WIE revision change.

No RustJava patch change.

## Acceptance

- actual PaintMidlet starts;
- Display.setCurrent requests redraw;
- test frontend pump forwards Redraw to pinned WIE;
- pinned EventQueue dispatches repaint;
- Canvas.paint executes without fault;
- screen.paint reaches M32 Capture Host;
- first frame is non-empty;
- session is Running;
- T005/T004/T003 verification chain remains green;
- all workspace quality gates pass.
