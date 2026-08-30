# M32 0.0.3-T002 FIX1 — ZIP magic literal correction

## Trigger

The deterministic JAD/JAR files and locked SHA-256 values were correct, and the adapter compiled.

The first fixture identity test failed at:

```text
assertion failed: FIRST_FRAME_BOOT_JAR.starts_with(b"PK\\x03\\x04")
```

## Root Cause

The generated Rust test accidentally used a doubly escaped byte string:

```rust
b"PK\\x03\\x04"
```

That represents ASCII backslash/x characters rather than ZIP local-file magic bytes.

## FIX1

The assertion is corrected to:

```rust
b"PK\x03\x04"
```

which matches:

```text
50 4B 03 04
```

## Scope

No fixture byte changes.
No JAD changes.
No JAR changes.
No Java source changes.
No production factory changes.
No dependency changes.
No Cargo.lock changes.

## Status

```text
0.0.3-T002 = FIX1 / IN_PROGRESS
```
