# M32 0.0.5 Audio Bundle A FIX1 — Rust 1.98 Clippy Gate

## Trigger

Bundle A compiled successfully with:

```text
cargo check -p m32-audio
exit 0
```

but the workspace `-D warnings` gate failed on:

```text
clippy::chunks-exact-to-as-chunks
```

at the fixed-size stereo PCM decode path.

## Root Cause

The implementation used:

```rust
samples.chunks_exact(2)
```

for a compile-time fixed stereo frame width.

Rust 1.98 Clippy requires the fixed-array API:

```rust
samples.as_chunks::<2>().0.iter()
```

under the project's `-D warnings` policy.

## FIX1

Only the stereo PCM iteration primitive is changed.

No audio math changes.

No normalization changes.

No resampling changes.

No mixer changes.

No GuestAudioHost behavior changes.

No dependency/Cargo.lock/WIE/RustJava changes are introduced by FIX1.

## Status

```text
0.0.5 Bundle A = FIX1 / IN_PROGRESS
```
