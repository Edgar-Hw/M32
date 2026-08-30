# M32 0.0.2-T010 FIX1 — test fixture binding correction

## Trigger

The T010 production library compiled successfully:

```text
cargo check -p m32-wie-adapter
exit 0
```

but test/all-target compilation failed with eight `E0425` errors.

The failures were limited to `#[cfg(test)]` code.

## Root Cause

While converting the new platform test helper from a large tuple into:

```text
RecordingPlatformFixture
```

a package post-processing replacement accidentally crossed test boundaries.

Existing `WieBasicHostBridge` tests locally bind:

```text
output
exit
vibration
```

but four assertions were incorrectly rewritten to:

```text
fixture.output
fixture.exit
fixture.vibration
```

where no `fixture` variable exists.

The new `WiePlatformAdapter` delegation test has the inverse situation: it binds:

```text
let fixture = recording_platform();
```

but four assertions were left using nonexistent local names:

```text
output
exit
vibration
```

## FIX1

Existing BasicHostBridge tests:

```text
fixture.output     -> output
fixture.exit       -> exit
fixture.vibration  -> vibration
```

New Platform delegation test:

```text
output             -> fixture.output
exit               -> fixture.exit
vibration          -> fixture.vibration
```

## Scope

No production `WiePlatformAdapter` behavior changes.

No host-service mapping changes.

No M32 public API changes.

No dependency changes.

No WIE revision changes.

No lint suppression added.

## Why ordinary cargo check passed

The bad bindings exist only under `#[cfg(test)]`.

Therefore:

```text
cargo check -p m32-wie-adapter
```

checked the normal library target and passed.

The problem is correctly detected by:

```text
cargo test -p m32-wie-adapter
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace --all-targets
```
