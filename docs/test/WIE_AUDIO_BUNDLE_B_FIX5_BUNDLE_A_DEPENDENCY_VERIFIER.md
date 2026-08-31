# M32 0.0.5 Audio Bundle B FIX5 — Bundle A Dependency Verifier Forward Compatibility

## Trigger

Bundle B runtime code passed its current test baseline, including:

```text
m32-audio       15/15
m32-wie-adapter 68/68
real J2ME MMAPI start/stop integration PASS
Input version-close recursive workspace gates PASS
```

The Bundle B verifier then failed while recursively invoking the older Bundle A verifier:

```text
m32-audio must have exactly one normal dependency in Bundle A; found 2
```

## Root Cause

Bundle A originally had one normal dependency:

```text
m32-emulator-api
```

Bundle B intentionally adds:

```toml
[target.'cfg(windows)'.dependencies]
cpal = "=0.18.2"
```

Cargo metadata reports target-specific dependencies with normal dependency kind plus a non-null
`target` expression.

The old Bundle A verifier counted both unconditional and target-specific normal dependencies, so a
valid later Bundle B state could never recursively revalidate Bundle A.

This was a verifier composition bug.

## FIX5

Bundle A now verifies exactly one **unconditional normal** dependency:

```text
m32-emulator-api
```

and requires it to remain workspace-local/path-based.

Target-specific dependencies are not counted as Bundle A core dependencies.

This does not make CPAL unverified. Bundle B separately requires:

```text
dependency name = cpal
exact version    = =0.18.2
Windows target   = Cargo target-specific dependency section
```

## What Is Not Changed

No runtime source is changed.

No change to:

```text
audio renderer
MMAPI fixture
WIE adapter
48kHz/f32/stereo contract
60ms target
80ms fade
MIDI baseline
CPAL version
Cargo.lock
```

## Status

```text
0.0.5 Bundle B = FIX5 / IN_PROGRESS
```
