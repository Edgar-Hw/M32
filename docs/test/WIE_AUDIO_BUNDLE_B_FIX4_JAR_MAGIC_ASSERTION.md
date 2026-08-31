# M32 0.0.5 Audio Bundle B FIX4 — JAR Magic Assertion

## Trigger

After FIX3:

```text
m32-audio 15/15 PASS
```

The recursive Bundle B verifier failed in:

```text
audio_mmapi_fixture_locks_guest_manager_player_contract
```

while the real runtime integration test already passed:

```text
real_j2me_mmapi_start_stop_reaches_m32_audio_host ... ok
```

## Root Cause

The fixture contract test accidentally used:

```rust
b"PK\\x03\\x04"
```

which encodes literal backslash/x characters.

A ZIP/JAR local-file header begins with actual bytes:

```text
50 4B 03 04
```

so the Rust byte literal must be:

```rust
b"PK\x03\x04"
```

## FIX4

Only this test assertion is corrected.

No changes to:

```text
JAD/JAR fixture bytes
fixture hashes
Java guest code
WIE MMAPI runtime path
GuestAudioHost path
m32-audio renderer
CPAL output
pause fade
MIDI baseline
dependencies
Cargo.lock
```

## Important Observation

The real Java MMAPI integration test passed before this assertion failure.

Therefore the failure was a verifier/test-literal bug, not an MMAPI runtime failure.

## Status

```text
0.0.5 Bundle B = FIX4 / IN_PROGRESS
```
