# M32 0.0.3-T005 — Deterministic Canvas/Paint Fixture

Status: IMPLEMENTATION BASELINE
Version: `0.0.3 First Frame`
Task: `0.0.3-T005`

## Purpose

T004 established the host-side first-frame capture point.

T005 introduces a deterministic M32-owned MIDlet whose UI contract is intentionally limited to the
small LCDUI subset implemented by the pinned WIE revision.

T005 locks fixture bytes and paint semantics only.

Actual guest ticking until the frame reaches `FirstFrameCaptureDisplayHost` is T006.

## Pinned LCDUI Contract Used

The fixture relies only on:

```text
Display.getDisplay(MIDlet) -> Display
Display.setCurrent(Displayable) -> void
protected Canvas::<init>() -> void
protected abstract Canvas.paint(Graphics) -> void
Graphics.setColor(int) -> void
Graphics.fillRect(int, int, int, int) -> void
```

Compile-time stubs use those exact Java descriptors.

Stub classes are not included in the runtime JAR.

Pinned WIE provides the runtime LCDUI implementations.

## Guest Classes

```text
m32.PaintMidlet
m32.PaintCanvas
```

`PaintMidlet.startApp()`:

```java
Display.getDisplay(this).setCurrent(new PaintCanvas());
System.out.println("M32_FIRST_FRAME_CANVAS_READY");
System.out.flush();
```

`PaintCanvas.paint(Graphics)`:

```java
graphics.setColor(0x0E1114);
graphics.fillRect(0, 0, 176, 220);

graphics.setColor(0xD14A36);
graphics.fillRect(0, 0, 16, 16);
```

## Locked Visual Pattern

Logical frame:

```text
176 x 220
```

Background:

```text
#0E1114
M32 BG0
```

Top-left marker:

```text
16 x 16
#D14A36
M32 RED
```

This pattern is deliberately simple and integer-only.

There is no text, font, image asset, interpolation, alpha blending, or external resource.

## Deterministic Fixture

JAD SHA-256:

```text
7fe94a7cb014c40367ee2139d25cf4e1c3cde37d8e9d73bcb1277a4a1f9efc33
```

JAR SHA-256:

```text
717063ece69306b9338da722d57f14815987c6a044f738da3b2e95373a9f8b5a
```

Class SHA-256:

```text
m32/PaintCanvas.class
87660a63443067de1c66d49c4246d60f575d0512a19c797ca235aa62dd1aa3c4

m32/PaintMidlet.class
85ebbc6796bd0c2dc9bd9192e931da3e9d714ef3e5d404e3543769267a73c706
```

Both classes are Java 8 classfile:

```text
minor = 0
major = 52
```

JAR properties:

```text
stored / no compression
fixed timestamp 2000-01-01 00:00:00
no manifest
exactly two runtime class entries
```

The JAD is the launch descriptor.

## Source Provenance

All guest code and compile-time stub code are M32-created synthetic test sources.

No commercial game code or asset is included.

## Tests

```text
first_frame_paint_fixture_locks_canvas_and_pixel_pattern_contract
first_frame_paint_fixture_constructs_ready_j2me_session
```

## Non-goals

T005 does not prove:

- `PaintMidlet.startApp()` executes;
- `Display.setCurrent()` succeeds at runtime;
- WIE dispatches a paint event;
- `PaintCanvas.paint()` executes;
- `present_rgba8()` receives a frame.

T006 proves the real end-to-end first-frame path.

## Acceptance

- deterministic JAD/JAR hashes are locked;
- both M32 guest classes exist in the JAR;
- paint source contract is fixed;
- explicit JAD main class is fixed;
- constructor returns a Ready M32 session;
- no new Rust dependency;
- no Cargo.lock change;
- T004/T003 verification chain remains green.
