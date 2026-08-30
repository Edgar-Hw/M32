# M32 Display Host Contract and WIE Screen Bridge

Status: LOCKED BASELINE
Task: `0.0.2-T005`
Emulator API schema: `1`

## 1. 목적

T005 defines the M32-owned display host contract required to adapt the pinned WIE `Screen`
interface without exposing WIE image/screen types to upper M32 layers.

This is an adapter contract Task, not the `0.0.3 First Frame` desktop rendering Task.

## 2. Pinned WIE Screen Surface

At WIE revision:

```text
f0513eb758c02736981f545ad030eed937d55f3e
```

the `Screen` contract exposes:

```text
resize(width, height) -> Result
request_redraw() -> Result
paint(&dyn Image)
width() -> u32
height() -> u32
```

The WIE `Image` contract exposes dimensions and per-pixel `Color` values.

## 3. M32 Display Contract

`m32-emulator-api` owns:

```text
DisplaySize
RgbaFrame
DisplayHost
DisplayHostError
```

Canonical frame representation for the adapter boundary:

```text
RGBA8
byte order per pixel: R, G, B, A
row order: top to bottom
pixel order: left to right
byte length: width * height * 4
```

`RgbaFrame::try_new` validates the byte length.

No WIE `Image`, `Color`, or `Screen` type appears in the M32 public API.

## 4. Reference Conversion Path

T005 uses a correctness-first conversion:

```text
for each y
    for each x
        WIE Image::get_pixel(x, y)
        -> append R,G,B,A
```

This deliberately ignores WIE `raw()` byte layout because WIE supports multiple concrete pixel
types and `bytes_per_pixel()` alone is insufficient to identify channel ordering.

The reference conversion may allocate/copy once per painted frame.

That is acceptable for Core Adapter correctness.

A later display/performance Task may add verified fast paths for known image formats, but the
canonical RGBA8 result must remain identical.

## 5. Resize and Redraw Errors

M32 display resize/redraw failures map to:

```text
wie_util::WieError::FatalError
```

because the pinned WIE `Screen` API requires `wie_util::Result`.

T005 therefore adds `wie_util` as a direct `m32-wie-adapter` dependency at the same locked WIE
revision already used by the dependency graph.

The WIE revision itself does not change.

## 6. Paint Failure Semantics

Pinned WIE `Screen::paint()` has no return value.

Therefore M32 cannot propagate a present failure back through WIE during paint.

T005 behavior:

- invalid/overflow frame dimensions -> structured `m32::display` error event, frame dropped;
- M32 `present_rgba8` failure -> structured `m32::display` error event, frame dropped.

No panic and no host process termination.

## 7. Threading

`DisplayHost: Send + Sync`.

`WieScreenAdapter` owns:

```text
Arc<dyn DisplayHost>
```

This matches WIE `Screen: Send + Sync` without forcing a concrete desktop renderer implementation
into the adapter crate.

## 8. Desktop Rendering Is Still Out of Scope

T005 does not:

- create a winit window;
- create a wgpu surface;
- upload a texture;
- scale a frame;
- implement Pixel Perfect/Smooth/LCD 2006 modes;
- display the first game frame.

Those belong to `0.0.3 First Frame` and later display Tasks.

## 9. Acceptance

- M32 `DisplaySize` RGBA8 length is deterministic;
- invalid frame byte lengths are rejected;
- `WieScreenAdapter` implements pinned WIE `Screen`;
- resize/redraw delegate to `DisplayHost`;
- a synthetic WIE image converts to exact canonical RGBA8 bytes;
- WIE display errors do not leak into `m32-emulator-api`;
- existing dependency boundary remains valid;
- all quality gates pass.
