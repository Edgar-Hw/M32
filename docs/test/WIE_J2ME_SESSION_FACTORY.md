# M32 WIE J2ME Session Factory

Status: LOCKED BASELINE
Task: `0.0.2-T011`
WIE revision: `f0513eb758c02736981f545ad030eed937d55f3e`

## 1. 목적

T011 creates the first actual WIE runtime-construction path in M32.

Input:

```text
WiePlatformHosts
JAR filename
JAR bytes
```

Output:

```text
Box<dyn m32_emulator_api::EmulatorSession>
```

The concrete WIE `J2MEEmulator` type remains private to the adapter implementation path.

## 2. Pinned J2ME Constructor

The locked `wie_j2me` crate publicly exports:

```text
J2MEEmulator
```

and provides:

```text
J2MEEmulator::from_jar(
    platform: Box<dyn Platform>,
    jar_filename: &str,
    jar: Vec<u8>,
) -> wie_util::Result<J2MEEmulator>
```

T011 calls this exact constructor.

## 3. Direct Dependency Pin

`m32-wie-adapter` gains a direct workspace dependency on:

```text
wie_j2me
```

using the exact same WIE Git revision as `wie_backend` and `wie_util`.

No floating WIE revision is introduced.

The lockfile is expected to change because the J2ME/JVM/MIDP dependency graph becomes directly
reachable from M32 for the first time.

## 4. M32 Session Creation Error

`m32-emulator-api` introduces generic, backend-independent:

```text
SessionCreateErrorCode::BackendLaunchFailed
EmulatorSessionCreateError
```

No `wie_util::WieError` or WIE constructor type enters the M32 API crate.

Constructor failures are converted at the adapter boundary.

## 5. Factory

`m32-wie-adapter` exposes:

```text
create_j2me_jar_session(...)
```

It:

1. consumes `WiePlatformHosts`;
2. constructs `WiePlatformAdapter`;
3. boxes it as `Box<dyn wie_backend::Platform>`;
4. transfers platform + filename + JAR bytes into `J2MEEmulator::from_jar`;
5. boxes the resulting emulator behind existing `WieSession`;
6. returns it as `Box<dyn EmulatorSession>` with state `Ready`.

## 6. What "Ready" Means

`Ready` means:

```text
the WIE J2ME emulator object was constructed and owns the supplied platform/JAR data
```

It does **not** mean:

```text
MIDlet main class successfully booted
first frame rendered
game proved playable
```

The pinned J2ME constructor schedules startup work. Actual guest execution occurs when the emulator
is ticked.

Therefore T011 deliberately does not call `tick()` on an empty synthetic JAR.

## 7. Constructor Smoke Fixture

The unit test uses:

```text
filename = synthetic-empty.jar
bytes = empty Vec
```

only to prove the synchronous constructor/ownership path.

This is not a valid playable JAR fixture and is never ticked.

A bootable deterministic fixture belongs to the following end-to-end Core Adapter smoke Task.

## 8. Boundary

`m32-emulator-api` remains WIE-independent.

`m32-desktop` still does not depend directly on WIE crates.

Only `m32-wie-adapter` owns:

```text
wie_j2me
wie_backend
wie_util
```

## 9. Non-Goals

T011 does not:

- boot a real MIDlet;
- call the new session's `tick()`;
- render the first game frame;
- add JAD launch;
- add input;
- add actual audio device output;
- implement format detection;
- import copyrighted game files.

## 10. Acceptance

- exact pinned `wie_j2me` resolves as a direct adapter dependency;
- `J2MEEmulator` implements pinned `wie_backend::Emulator`;
- M32 Platform is transferred into the pinned J2ME constructor;
- JAR filename and bytes are accepted by the factory path;
- returned M32 session starts in `Ready`;
- constructor errors map to stable M32 session-create error code;
- no WIE type enters `m32-emulator-api`;
- all quality gates pass.
