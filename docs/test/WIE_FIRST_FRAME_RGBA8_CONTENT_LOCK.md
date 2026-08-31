# M32 0.0.3-T007 — RGBA8 Frame Dimension/Content Lock

Status: IMPLEMENTATION BASELINE
Version: `0.0.3 First Frame`
Task: `0.0.3-T007`

## Purpose

T006 proved that a real M32-owned Java Canvas can produce a frame that reaches the M32 display host.

T007 converts that visual smoke into an exact framebuffer contract.

The captured first frame must no longer be merely non-empty.

It must be exactly the frame that `m32.PaintCanvas` is defined to paint.

## Locked Frame

Dimensions:

```text
176 x 220
```

Pixel format:

```text
RGBA8
4 bytes per pixel
```

Exact framebuffer byte length:

```text
176 * 220 * 4 = 154880 bytes
```

Total pixels:

```text
38720
```

## Locked Pattern

Marker:

```text
x = 0..15
y = 0..15
RGBA = [0xD1, 0x4A, 0x36, 0xFF]
#D14A36FF
```

Marker count:

```text
16 * 16 = 256 pixels
```

Background:

```text
all remaining coordinates
RGBA = [0x0E, 0x11, 0x14, 0xFF]
#0E1114FF
```

Background count:

```text
38720 - 256 = 38464 pixels
```

## Exact Coordinate Checks

The test explicitly locks marker corners:

```text
(0,0)
(15,0)
(0,15)
(15,15)
```

as M32 RED.

It explicitly locks immediate outside-boundary samples:

```text
(16,0)
(0,16)
```

as M32 BG0.

Bottom-right:

```text
(175,219)
```

must also be M32 BG0.

## Full-frame Assertion

T007 does not rely only on sample pixels.

Every pixel in the 176x220 first frame is iterated.

For each `(x,y)`:

```text
x < 16 && y < 16
-> RED

otherwise
-> BG0
```

Any unexpected coordinate reports the exact failing `(x,y)`.

## Runtime Path

The frame still comes from the real T006 runtime path:

```text
PaintMidlet
-> Display.setCurrent
-> redraw
-> Event::Redraw
-> EventQueue
-> PaintCanvas.paint
-> WIE screen.paint
-> WieScreenAdapter
-> FirstFrameCaptureDisplayHost
```

No expected frame is injected into the host.

## First-frame Immutability

The test also verifies that:

```text
display.first_frame() == captured frame
```

after exact content validation.

## Test

```text
first_frame_paint_fixture_locks_exact_rgba8_dimensions_and_content
```

## Non-goals

T007 does not add timeout/failure semantics.

Those belong to:

```text
0.0.3-T008
First-frame timeout/failure boundary
```

## Dependencies

No new dependency.

No Cargo.lock change.

No WIE/RustJava change.

No guest fixture change.

## Expected Test Count

T006:

```text
55 adapter tests
```

T007 adds one:

```text
56 adapter tests
```

Minimum workspace unit tests after T007:

```text
99
```
