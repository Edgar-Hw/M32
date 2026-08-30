# M32 0.0.3-T002 — Deterministic Bootable MIDlet Fixture

Status: IMPLEMENTATION BASELINE
Version: `0.0.3 First Frame`
Task: `0.0.3-T002`
WIE revision: `f0513eb758c02736981f545ad030eed937d55f3e`

## Purpose

T002 introduces the first M32-owned J2ME artifact containing a real Java class that the pinned
RustJava JVM can load as a MIDlet.

T002 validates fixture identity and constructor ownership only.

Actual ticking/boot success is intentionally deferred to T003 so fixture packaging failures and
runtime failures remain distinguishable.

## MIDlet Contract

Pinned WIE's launcher:

```text
1. creates the configured main class with constructor ()V;
2. passes it as javax.microedition.midlet.MIDlet;
3. invokes startApp()V virtually.
```

Pinned MIDP `MIDlet` provides a protected `()V` constructor and declares protected abstract
`startApp()V`.

Therefore the minimal fixture class is:

```java
package m32;

import javax.microedition.midlet.MIDlet;

public final class FirstFrameMidlet extends MIDlet {
    public FirstFrameMidlet() {
        super();
    }

    protected void startApp() {
    }
}
```

No LCDUI call is made yet.

## Fixture Files

```text
crates/m32-wie-adapter/test-fixtures/j2me-first-frame-boot.jad
crates/m32-wie-adapter/test-fixtures/j2me-first-frame-boot.jar
crates/m32-wie-adapter/test-fixtures/src/m32/FirstFrameMidlet.java
crates/m32-wie-adapter/test-fixtures/src/javax/microedition/midlet/MIDlet.java
```

The `javax.microedition.midlet.MIDlet` Java source is compile-time-only scaffolding. It is **not**
included in the JAR. At runtime the superclass is provided by pinned WIE MIDP.

## Classfile Baseline

The committed `.class` is generated with:

```text
javac --release 8 -g:none
```

Expected classfile:

```text
minor = 0
major = 52
```

Java 8 classfile major `52` is within the pinned RustJava parser's accepted range.

## Determinism

The JAR:

- stores the class without compression;
- contains exactly `m32/FirstFrameMidlet.class`;
- uses a fixed ZIP timestamp `2000-01-01 00:00:00`;
- contains no manifest;
- relies on the explicit T001 JAD launch path.

Hashes:

```text
JAD SHA-256   a30e92605f738bdca0eeb2f7c694b87aa21fbf212215c652287fb50c8e9f745d
CLASS SHA-256 27738f217b6180ba72d91aaaa5c063d32f100554c8b3d74751ff3ff359b2b7b3
JAR SHA-256   9ff5c3a86d913f7f49453312773af6b1cc43f595674e1206bcb64268dc573b3c
```

## Why No Manifest

T001 established the explicit `from_jad_jar` path and avoids dependence on the known JAR-only
manifest startup branch.

The deterministic First Frame fixture therefore uses the JAD as the launch descriptor and keeps the
JAR focused on class content.

## Tests

T002 unit tests lock:

- ZIP/JAR local-file signature;
- exact class entry name;
- class internal name;
- Java 8 classfile magic/version sequence;
- exact JAD `MIDlet-1`;
- constructor path returns `SessionState::Ready`.

## Non-goals

T002 does not:

- call `tick()`;
- prove the class is successfully instantiated by RustJava;
- prove `startApp()` executes;
- reach `Running`;
- create LCDUI objects;
- produce a frame.

Those begin at T003.

## Acceptance

- deterministic JAD/JAR exist;
- hashes match locked values;
- source provenance exists;
- JAR contains only M32 synthetic guest code;
- classfile is Java 8 major 52;
- JAD points to `m32.FirstFrameMidlet`;
- actual pinned JAD+JAR constructor returns Ready;
- no new Rust dependency;
- previous verifier chain remains green.
