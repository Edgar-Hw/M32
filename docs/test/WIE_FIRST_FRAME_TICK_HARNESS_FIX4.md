# M32 0.0.3-T006 FIX4 — Rust 1.98 Clippy cleanup

## Trigger

All T006 runtime and regression tests passed, including:

```text
m32-wie-adapter 55/55
m32-emulator-api 29/29
Tick-until-first-frame verifier PASS
workspace tests PASS
```

The remaining gate failure was:

```text
clippy::chunks_exact_to_as_chunks
```

in the optional test-only BMP preview exporter.

## Fix

Replace:

```rust
frame.pixels.chunks_exact(4)
```

with:

```rust
let (rgba_pixels, remainder) = frame.pixels.as_chunks::<4>();
debug_assert!(remainder.is_empty());

for rgba in rgba_pixels {
    ...
}
```

The exporter already asserts the exact `176 * 220 * 4` RGBA8 byte length before this block, so the
remainder is expected to be empty.

## Scope

No runtime emulation behavior change.

No guest fixture change.

No frame-capture behavior change.

No WIE/RustJava change.

No dependency/Cargo.lock change.

Only Rust 1.98 Clippy compliance for the optional BMP preview helper.

## Status

```text
0.0.3-T006 = FIX4 / IN_PROGRESS
```
