# M32 WIE Host Service Inventory

Status: LOCKED BASELINE
Task: `0.0.2-T004`
Emulator API schema: `1`

## 1. 목적

The pinned WIE backend cannot create a useful emulator runtime by itself. Its `Platform` contract
requires host-provided services.

T004 records those requirements as M32-owned service identifiers before any concrete host adapter is
implemented.

This prevents the M32 application architecture from importing `wie_backend::Platform` directly.

## 2. Pinned WIE Platform Surface

At WIE revision:

```text
f0513eb758c02736981f545ad030eed937d55f3e
```

`wie_backend::Platform` requires these methods:

```text
screen()
now()
database_repository()
filesystem()
audio_sink()
write_stdout()
write_stderr()
exit()
vibrate()
```

T004 maps them to M32-owned `HostServiceKind` values.

## 3. Locked M32 Host Service IDs

Order is intentional and stable for diagnostics/evidence:

```text
display
clock
database
filesystem
audio
stdout
stderr
exit
vibration
```

Mapping:

```text
WIE screen()              -> Display
WIE now()                 -> Clock
WIE database_repository() -> Database
WIE filesystem()          -> Filesystem
WIE audio_sink()          -> Audio
WIE write_stdout()        -> Stdout
WIE write_stderr()        -> Stderr
WIE exit()                -> Exit
WIE vibrate()             -> Vibration
```

## 4. Backend Requirement Contract

`EmulatorBackend` gains:

```text
required_host_services()
```

A backend declares what host services are required before a concrete session can be created.

`WieBackendAdapter` returns exactly the nine services above.

Synthetic/non-WIE backends may return a different list, including an empty list.

## 5. Why T004 Does Not Implement Platform Yet

Each WIE platform surface has different ownership and lifetime constraints:

- display owns framebuffer/redraw semantics;
- clock supplies epoch-millisecond time;
- database is asynchronous and app-scoped;
- filesystem is asynchronous and AID-scoped;
- audio creates sink objects and accepts structured audio commands;
- process output must be routed without leaking arbitrary bytes into normal INFO logs;
- exit is a guest-requested lifecycle signal, not host process termination;
- vibration maps to a host effect that may have no physical actuator.

Implementing all of these as one large Task would freeze several unrelated M32 contracts at once.

T004 therefore locks only the inventory.

## 6. Compile Probe

`m32-wie-adapter` contains a compile-only test probe against the pinned `wie_backend::Platform`
surface.

It references all nine WIE methods without constructing a runtime.

If the pinned source or future approved WIE revision changes that surface, the test will fail at
compile time and require an explicit adapter update.

## 7. WIE Type Leakage Rule

`HostServiceKind` belongs to `m32-emulator-api`.

No WIE `Platform`, `Screen`, `Filesystem`, `DatabaseRepository`, `AudioSink`, or `Instant` type is
added to the public M32-facing API.

## 8. Non-Goals

T004 does not:

- implement `wie_backend::Platform`;
- create a screen;
- read/write guest files;
- create databases;
- play audio;
- perform vibration;
- terminate the host process;
- instantiate J2ME/SKT/LGT emulators.

Those surfaces are implemented incrementally in later Core Adapter Tasks.
