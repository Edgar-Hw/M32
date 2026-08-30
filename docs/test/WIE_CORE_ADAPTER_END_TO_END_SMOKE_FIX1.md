# M32 0.0.2-T012 FIX1 — contain upstream WIE tick panics

## Trigger

The first real J2ME tick smoke reached the pinned JVM/MIDP runtime and panicked:

```text
called `Result::unwrap()` on an `Err` value:
JavaException(ClassInstance(java/io/FileNotFoundException))
```

The panic originated inside pinned:

```text
wie_j2me/src/emulator.rs
```

during the JAR manifest-property startup path.

The synthetic JAR itself was found and its manifest was parsed. The failure happened after resource
loading, while WIE propagated manifest properties through `java/lang/System.setProperty`.

## Root Cause

Pinned WIE has several `unwrap()` calls in the JAR-only startup path.

A Java exception at one of those sites becomes a Rust panic instead of `wie_util::Result::Err`.

The existing `WieSession::tick()` only mapped normal `Result::Err`, so an upstream panic escaped the
adapter boundary.

## Decision

The M32 WIE adapter owns a panic containment boundary around:

```text
self.emulator.tick()
```

using Rust unwind containment.

Outcomes:

```text
Ok(Ok(()))
-> Running

Ok(Err(wie_error))
-> Faulted
-> existing BackendTickFailed mapping

Err(panic_payload)
-> Faulted
-> BackendTickFailed
-> fixed sanitized message
```

Sanitized panic message:

```text
pinned WIE backend panicked during tick
```

The panic payload is not exposed.

## Logging

The adapter emits:

```text
target = m32::emulator
event = wie_tick_panicked
```

without serializing the panic payload or guest data.

## Why the fixture is not changed

Changing the manifest to avoid the pinned WIE panic would make the smoke test pass while leaving a
known crash path in the real emulator boundary.

T012 intentionally keeps the original deterministic fixture and converts the observed upstream panic
into a stable M32 session failure.

## Regression Tests

A direct synthetic `PanickingWieEmulator` test verifies:

```text
panic
-> BackendTickFailed
-> fixed sanitized message
-> SessionState::Faulted
```

The real J2ME smoke then verifies the same containment boundary against the pinned WIE/RustJava
runtime.

## Panic-hook note

`catch_unwind` contains control flow, but Rust's panic hook runs before the unwind is caught.

Therefore debug/test output may still print the upstream panic line. The acceptance condition is that
the test continues and returns the stable M32 error instead of aborting the test process.

## Scope

No WIE source edit.

No WIE revision change.

No RustJava revision change.

No fixture-byte change.

No new dependency.

No M32 public API change.

## Status

```text
0.0.2-T012 = FIX1 / IN_PROGRESS
```
