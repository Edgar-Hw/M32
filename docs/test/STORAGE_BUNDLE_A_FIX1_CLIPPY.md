# M32 0.0.6 Storage Bundle A FIX1 — Rust 1.98 Clippy Alignment

## Trigger

The initial Storage Bundle A result was:

```text
cargo check -p m32-storage = PASS
cargo test -p m32-storage  = 14/14 PASS
cargo clippy -p m32-storage --all-targets -- -D warnings = FAIL
```

The failures were Clippy-only:

```text
clippy::redundant_closure
clippy::manual_noop_waker
```

## FIX1

Production code replaces the redundant `map_err` closure with the associated function directly.

The test harness replaces the custom `NoopWake` implementation with the Rust standard library's `Waker::noop()`.

## Scope

No changes to SQLite schema, WAL/FK/busy-timeout policy, RMS semantics, record IDs, usage accounting, guest filesystem behavior, isolation, traversal rejection, restart persistence, dependencies, WIE, or RustJava.

## Expected Result

```text
cargo clippy -p m32-storage --all-targets -- -D warnings
exit 0

cargo test -p m32-storage
14 passed
0 failed
```
