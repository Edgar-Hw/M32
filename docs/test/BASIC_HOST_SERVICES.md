# M32 Basic Host Service Contracts

Status: LOCKED BASELINE
Task: `0.0.2-T006`
Emulator API schema: `1`

## 1. 목적

T006 defines M32-owned contracts for the synchronous non-storage/non-audio host services required
by the pinned WIE `Platform`.

Covered surfaces:

```text
clock
stdout
stderr
exit
vibration
```

Display was bridged in T005.
Filesystem, Database, and Audio remain separate Tasks.

## 2. Pinned WIE Mapping

At the locked WIE revision:

```text
Platform::now()                 -> ClockHost::epoch_millis()
Platform::write_stdout(&[u8])   -> GuestOutputHost::write_stdout()
Platform::write_stderr(&[u8])   -> GuestOutputHost::write_stderr()
Platform::exit()                -> ExitHost::request_exit()
Platform::vibrate(u64, u8)      -> VibrationHost::vibrate()
```

## 3. Clock Contract

M32 exposes:

```text
ClockHost::epoch_millis() -> u64
```

The value is milliseconds since the Unix epoch.

The WIE adapter converts it with:

```text
wie_backend::Instant::from_epoch_millis(...)
```

No wall-clock sampling is performed by `m32-wie-adapter` itself.

A concrete host clock is supplied later by the runtime/desktop layer.

## 4. Guest Output Contract

M32 exposes raw byte channels:

```text
GuestOutputHost::write_stdout(&[u8])
GuestOutputHost::write_stderr(&[u8])
```

The API deliberately does not require UTF-8.

Legacy guest runtimes may emit arbitrary byte sequences.

T006 does not route these bytes into normal M32 INFO logs. A later concrete host implementation
defines buffering/sanitization/logging policy.

## 5. Exit Contract

WIE `Platform::exit()` is mapped to:

```text
ExitHost::request_exit()
```

This is a **guest runtime exit request**.

It is not defined as:

```text
std::process::exit(...)
```

and must not directly terminate the M32 desktop process.

A later runtime orchestration Task decides how a guest exit transitions session state.

## 6. Vibration Contract

M32 preserves the pinned WIE values exactly:

```text
duration_ms: u64
intensity: u8
```

T006 does not reinterpret, normalize, clamp, or synthesize these values.

A desktop without a vibration-capable device may later implement this as a no-op or optional effect,
but the adapter boundary preserves the guest request.

## 7. Adapter Bridge

`m32-wie-adapter` introduces internal:

```text
WieBasicHostBridge
```

It owns:

```text
Arc<dyn ClockHost>
Arc<dyn GuestOutputHost>
Arc<dyn ExitHost>
Arc<dyn VibrationHost>
```

and provides adapter-owned delegation behavior for future `WiePlatformAdapter` assembly.

Its public methods expose only M32-owned primitive values and callbacks. In particular, the bridge
returns epoch milliseconds rather than `wie_backend::Instant`, so no WIE type is added to the
adapter's M32-facing helper surface.

The pinned WIE `Instant::from_epoch_millis(...)` conversion is compile-tested from that value.

## 8. Non-Goals

T006 does not:

- implement the complete WIE `Platform`;
- sample the real Windows clock;
- print guest bytes to the Windows console;
- terminate the desktop process;
- trigger a physical vibration device;
- implement filesystem/database/audio services;
- alter session state on guest exit.

## 9. Acceptance

- `ClockHost` preserves full `u64` epoch milliseconds;
- guest stdout/stderr preserve arbitrary raw bytes;
- exit is represented as a request callback;
- vibration values are preserved exactly;
- WIE `Instant` receives the exact epoch-millisecond value;
- existing WIE/M32 dependency boundary remains valid;
- all quality gates pass.
