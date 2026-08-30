# M32 0.0.3-T003 — Positive Boot-to-Running Smoke

Status: IMPLEMENTATION BASELINE
Version: `0.0.3 First Frame`
Task: `0.0.3-T003`

## Purpose

T003 proves that an M32-owned J2ME class is not merely accepted by the constructor but is actually
loaded and executed by the pinned RustJava/WIE runtime.

A `SessionState::Running` assertion alone is insufficient evidence because the M32 session wrapper
marks the session Running after a successful backend tick.

T003 therefore requires a guest-produced observable sentinel from inside `MIDlet.startApp()`.

## Guest

Class:

```text
m32.RunningMidlet
```

`startApp()`:

```java
protected void startApp() {
    System.out.println("M32_FIRST_FRAME_BOOT_OK");
    System.out.flush();
}
```

The sentinel is ASCII-only so the WIE JVM's configured EUC-KR encoding cannot alter the marker
bytes.

## Why stdout

Pinned RustJava initializes `System.out` from runtime stdout.

Pinned `wie_jvm_support` maps that stdout file descriptor to:

```text
System.platform().write_stdout(...)
```

which reaches the existing M32 `GuestOutputHost`.

Therefore observing the marker at `RecordingOutput.stdout` proves this full path:

```text
JAD main class
-> RustJava class load
-> RunningMidlet::<init>
-> MIDlet::<init>
-> Launcher.startMIDlet
-> virtual MIDlet.startApp()
-> java.lang.System.out
-> java.io.PrintStream
-> WIE JVM stdout file
-> WIE Platform.write_stdout
-> M32 GuestOutputHost
```

## Fixture

```text
j2me-first-frame-running.jad
j2me-first-frame-running.jar
src/m32/RunningMidlet.java
```

T002's original fixture is left unchanged so all T002 hash locks and verifier behavior remain valid.

Locked hashes:

```text
JAD   e02fba5850f154913d1ed45d845c63f4cad53a2f1d66d348f5046a83c56a1ae7
CLASS f8050f4bc46b98c32c77f17f02d20cf025b3488feb913b87d0471a30d6fbc659
JAR   2521a16329f92ec6eaf07a2d9bc379713261c00146a722c14b5dd0aab5bb465d
```

Classfile:

```text
minor = 0
major = 52
```

## Bounded Boot

The smoke allows at most:

```text
512 ticks
```

to observe the sentinel.

Every tick before the sentinel must return `Ok(())`.

Any M32 fault, WIE error, or contained WIE panic fails the positive boot smoke.

## Required Result

When the sentinel appears:

```text
SessionState::Running
```

must be true.

## Non-goals

T003 does not:

- create a Display;
- create a Canvas;
- call paint;
- emit a framebuffer;
- open a native window.

Those begin after positive guest boot has been established.

## Acceptance

- fixture identity is locked;
- real JAD/JAR runtime is ticked;
- guest `startApp()` sentinel reaches M32 stdout;
- no startup error/panic occurs before sentinel;
- M32 session is Running;
- previous T002/T001/Core Adapter verifiers remain green;
- no new Rust dependency.
