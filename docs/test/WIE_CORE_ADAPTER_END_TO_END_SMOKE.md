# M32 Core Adapter End-to-End J2ME Tick Smoke

Status: LOCKED BASELINE
Task: `0.0.2-T012`
WIE revision: `f0513eb758c02736981f545ad030eed937d55f3e`
RustJava revision: `ba5797b8eb4cf376fdd63129903d319d1d7acf98`

## 1. 목적

T012 closes `0.0.2 Core Adapter` by exercising the complete J2ME construction and tick path through
the stable M32 session boundary.

This Task is deliberately a deterministic failure-path smoke test.

Successful MIDlet rendering belongs to `0.0.3 First Frame`.

## 2. Synthetic Fixture

M32 owns a tiny synthetic JAR:

```text
crates/m32-wie-adapter/test-fixtures/j2me-core-smoke-missing-main.jar
```

It contains only:

```text
META-INF/MANIFEST.MF
```

with:

```text
Manifest-Version: 1.0
MIDlet-Name: M32 Core Smoke
```

and intentionally has no:

```text
MIDlet-1
```

entry.

The fixture contains no third-party game code or copyrighted game assets.

SHA-256:

```text
690642fc8ce47f9d74d8898744709c219a56db98c0585289f8ebc3c24b1d0556
```

The JAR is stored without compression and with a fixed ZIP timestamp to keep the bytes deterministic.

## 3. Why Missing Main

Pinned WIE `J2MEEmulator::from_jar(...)` constructs a `System`, installs the supplied JAR as a
virtual file, and schedules startup work.

Actual startup occurs during subsequent `tick()` calls.

When launching from JAR without an explicitly supplied JAD main class, pinned WIE reads:

```text
META-INF/MANIFEST.MF
```

The pinned revision currently reaches its manifest-property propagation path before the intended
missing-main error. During that path, a Java exception can be unwrapped inside WIE and become a Rust
panic.

T012 FIX1 intentionally keeps this fixture unchanged because that panic is a real upstream runtime
behavior that the M32 adapter must contain.

This fixture therefore exercises substantially more than the T011 constructor smoke:

```text
M32 hosts
-> WiePlatformAdapter
-> J2MEEmulator::from_jar
-> WieSession
-> EmulatorSession::tick
-> WIE System::tick
-> RustJava JVM / MIDP startup
-> JAR manifest read
-> pinned WIE manifest-property startup path
-> upstream WIE panic
-> WieSession panic containment
-> stable BackendTickFailed mapping
-> SessionState::Faulted
```

## 4. Bounded Tick Policy

The test allows at most:

```text
512 ticks
```

to reach the expected fault boundary.

The bound prevents accidental infinite tests if the pinned scheduler behavior changes.

The test does not require the failure to occur on an exact tick number.

## 5. Required M32 Result

Whether pinned WIE returns a normal backend error or panics while executing the guest tick path:

```text
EmulatorSession::tick()
```

must contain the failure and return:

```text
SessionErrorCode::BackendTickFailed
```

and the session state must become:

```text
SessionState::Faulted
```

For ordinary WIE `Result::Err` values, the existing error mapping remains unchanged.

For an upstream panic, M32 deliberately does not expose the panic payload. The stable message is:

```text
pinned WIE backend panicked during tick
```

The adapter emits only a structured `m32::emulator` panic event and transitions the session to
`Faulted`.

The Rust panic hook may still print diagnostic text in test/debug output before `catch_unwind`
returns control to M32. That diagnostic output does not mean the test escaped the containment
boundary.

## 6. Scope Boundary

T012 proves:

- exact vendored JVM graph is executable;
- J2ME constructor ownership path works;
- WIE scheduled startup actually runs under M32 ticks;
- guest JAR virtual file access reaches the runtime;
- runtime failure cannot escape the adapter as an uncaught unwind;
- ordinary WIE errors and upstream WIE panics both become stable M32 failures;
- session lifecycle transitions Ready -> Faulted deterministically.

T012 does not prove:

- a valid MIDlet boots;
- a first frame is rendered;
- guest input works;
- sound reaches a device.

Those are subsequent version responsibilities.

## 7. Dependency Policy

No new dependency is added in T012.

The fixture SHA is verified by PowerShell using the operating-system SHA-256 facility.

## 8. Acceptance

- fixture exists and SHA-256 matches;
- J2ME session constructs in Ready state;
- real WIE runtime is ticked;
- bounded smoke reaches M32 BackendTickFailed;
- session becomes Faulted;
- all previous Core Adapter verifiers still pass;
- all workspace quality gates pass.
