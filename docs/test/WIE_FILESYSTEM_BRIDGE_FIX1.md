# M32 0.0.2-T007 FIX1 — Clippy test helper type complexity

## Trigger

T007 functional validation passed:

- `m32-emulator-api`: 18 tests passed
- `m32-wie-adapter`: 21 tests passed
- WIE filesystem bridge verifier: passed
- workspace tests: passed
- workspace all-target check: passed
- `git diff --check`: passed

The remaining quality gate failure was:

```text
clippy::type-complexity
RecordingFilesystemHost.write:
Mutex<Option<(String, String, usize, Vec<u8>)>>
```

## Correction

FIX1 replaces test-only tuple recording values with explicit test helper structs:

```text
RecordedWrite
- aid
- path
- offset
- data

RecordedTruncate
- aid
- path
- len
```

`RecordingFilesystemHost` now stores:

```text
Mutex<Option<RecordedWrite>>
Mutex<Option<RecordedTruncate>>
```

## Scope

No production filesystem behavior changes.

No M32 public API changes.

No dependency changes.

No WIE revision changes.

No filesystem fallback/error semantics change.
