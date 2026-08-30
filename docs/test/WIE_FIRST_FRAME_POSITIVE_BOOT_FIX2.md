# M32 0.0.3-T003 FIX2 — isolate WIE virtual JAR vs RustJava class-loader failure

## Trigger

T003 FIX1 corrected the test host's filesystem semantics:

```text
unknown archive path
exists -> false
size   -> None
read   -> None
```

The regression test passed.

The real J2ME positive-boot smoke nevertheless failed with the same stable runtime error:

```text
java.lang.NoClassDefFoundError: m32/RunningMidlet
```

Therefore the original test-host metadata shadowing was a real bug, but it was not the complete root
cause of the class-loading failure.

## What is already proven

The following are now independently proven:

```text
JAD hash / MIDlet-1       correct
JAR hash                  correct
JAR entry name            m32/RunningMidlet.class
class internal name       m32/RunningMidlet
classfile major           52
constructor path          Ready
unknown persistent path   does not claim JAR
```

The final `NoClassDefFoundError` means the Java class-loader returned no class for
`m32/RunningMidlet`. It does not by itself distinguish between:

- classpath JAR metadata not being queried;
- the JAR not being opened/read through the overlay;
- the JAR entry resource lookup returning no entry.

## FIX2 Strategy

FIX2 is diagnostic and test-only.

It does not alter:

- production adapter behavior;
- JAD/JAR/class bytes;
- WIE source;
- RustJava source;
- dependency graph.

### 1. Real WIE overlay round-trip test

New test:

```text
first_frame_running_jar_round_trips_through_real_wie_overlay
```

It builds the actual:

```text
WiePlatformAdapter
-> wie_backend::System
-> FilesystemOverlay
```

then adds the exact 522-byte T003 JAR as a virtual file.

It verifies:

```text
exists -> true
size   -> exact JAR byte length
read   -> exact full JAR bytes
```

This determines whether M32 Platform + pinned WIE overlay can faithfully expose the archive before
RustJava is involved.

### 2. Boot filesystem call trace

The positive boot smoke now retains the `RecordingFilesystemHost` observer.

If boot fails, the panic message includes the exact sequence:

```text
filesystem calls: [...]
```

Examples of interpretation:

```text
no size call for j2me-first-frame-running.jar
-> RustJava classpath/URL layer never queried the JAR

size call but no read-related access
-> metadata succeeded but JAR connection/resource stage did not open the archive

size plus archive read path
-> archive reached RustJava; investigate resource/entry lookup next
```

Note that WIE's virtual fallback reads do not invoke the persistent host's `read()` when
`exists()` is false. Therefore the host trace is interpreted together with the explicit overlay
round-trip test, not by persistent `read()` calls alone.

## Status

```text
0.0.3-T003 = FIX2 / DIAGNOSTIC / IN_PROGRESS
```

Do not commit T003 until the positive boot smoke succeeds.
