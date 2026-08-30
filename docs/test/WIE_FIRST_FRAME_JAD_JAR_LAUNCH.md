# M32 0.0.3-T001 — Explicit JAD + JAR Launch Path

Status: IMPLEMENTATION BASELINE
Version: `0.0.3 First Frame`
Task: `0.0.3-T001`
WIE revision: `f0513eb758c02736981f545ad030eed937d55f3e`

## Purpose

The First Frame phase begins by adding a positive J2ME launch path that supplies the main MIDlet
class explicitly through JAD metadata.

T012 proved that the pinned WIE JAR-only startup path can panic while re-reading and propagating JAR
manifest properties.

Pinned WIE also exposes:

```text
J2MEEmulator::from_jad_jar(
    platform,
    jad,
    jar_filename,
    jar
)
```

The JAD parser extracts `MIDlet-1` synchronously, and the resulting main-class name is supplied to
the scheduled startup task. The later startup path therefore uses the explicit main class instead of
the JAR-only manifest main-class discovery branch.

## New Factory

`m32-wie-adapter` adds:

```text
create_j2me_jad_jar_session(...)
```

Inputs:

```text
WiePlatformHosts
JAD bytes
JAR filename
JAR bytes
```

Output:

```text
Result<Box<dyn EmulatorSession>, EmulatorSessionCreateError>
```

The concrete WIE `J2MEEmulator` still does not escape the adapter crate.

## Why This Is First Frame T001

A deterministic successful MIDlet fixture needs a stable entry-point path before any Canvas/paint
behavior can be tested.

The First Frame sequence therefore starts with:

```text
T001 explicit JAD+JAR launch path
T002 deterministic bootable MIDlet fixture
T003 positive boot-to-Running smoke
...
```

This keeps the known JAR-only panic isolated from the positive First Frame fixture.

## Construction Semantics

The factory:

1. consumes M32 host services;
2. constructs `WiePlatformAdapter`;
3. boxes it as pinned WIE `Platform`;
4. passes JAD bytes, JAR filename and JAR bytes to pinned `from_jad_jar`;
5. wraps the concrete J2ME emulator in `WieSession`;
6. returns `SessionState::Ready`.

T001 does not tick the guest yet.

## Synthetic JAD

Tests use M32-owned descriptor bytes:

```text
MIDlet-Name: M32 First Frame Smoke
MIDlet-Version: 1.0.0
MIDlet-Vendor: M32
MIDlet-1: M32 First Frame,,m32.FirstFrameMidlet
```

The JAR bytes may remain empty in T001 because the constructor only needs to schedule startup.
A real synthetic class is introduced in T002.

## Non-goals

T001 does not:

- create a Java class fixture;
- execute the MIDlet;
- reach Running state;
- create a Canvas;
- call paint;
- produce an RGBA frame;
- add a host window.

## Acceptance

- direct pinned `from_jad_jar` constructor path is compiled;
- JAD/JAR factory returns an M32 `Ready` session;
- no WIE type leaks into `m32-emulator-api`;
- no dependency or lockfile change;
- previous Core Adapter verification remains green;
- all quality gates pass.
