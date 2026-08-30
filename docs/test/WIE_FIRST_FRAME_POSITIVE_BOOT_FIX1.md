# M32 0.0.3-T003 FIX1 — test filesystem must not shadow WIE virtual JAR metadata

## Trigger

T003 fixture identity passed, but the first real positive boot failed:

```text
Fatal error:
java.lang.NoClassDefFoundError: m32/RunningMidlet
    at net/wie/Launcher.start(Ljava/lang/String;)V
```

The failure was a normal WIE error and was safely mapped to:

```text
SessionErrorCode::BackendTickFailed
```

No Rust panic escaped the adapter.

## Root Cause

The JAD and JAR hashes were correct.

The JAR contained:

```text
m32/RunningMidlet.class
```

and the class identity test passed.

The problem was the test-only `RecordingFilesystemHost`.

Its old behavior was inconsistent:

```text
exists(path)
-> true only for save/state.bin

size(path)
-> Some(6) for every path
```

Pinned WIE stores the supplied application JAR in `FilesystemOverlay` as a virtual file.

The overlay's `size(path)` resolution order is:

```text
1. persistent Platform::filesystem().size(aid, path)
2. virtual-file size fallback
```

Because the test host incorrectly returned `Some(6)` for:

```text
j2me-first-frame-running.jar
```

the actual 522-byte virtual JAR was reported to RustJava as a six-byte file.

RustJava's URL/JAR class loader therefore could not discover:

```text
m32/RunningMidlet.class
```

and the launcher ultimately received `NoClassDefFoundError`.

## FIX1

`RecordingFilesystemHost` now behaves coherently:

```text
save/state.bin
exists -> true
size   -> Some(6)
read   -> abcdef

all unrelated paths
exists -> false
size   -> None
read   -> None
```

This allows WIE's virtual filesystem layer to remain authoritative for synthetic JAR files.

## Regression Test

Added:

```text
recording_filesystem_does_not_claim_unknown_virtual_archive_paths
```

It locks the test host behavior for:

```text
aid  = M32 Running Smoke
path = j2me-first-frame-running.jar
```

Expected:

```text
exists -> false
size   -> None
read   -> None
```

## Scope

This is a `#[cfg(test)]` fixture correction.

No production `WieFilesystemAdapter` behavior changes.

No WIE source changes.

No RustJava source changes.

No JAD/JAR/class bytes change.

No dependency or Cargo.lock change.

## Status

```text
0.0.3-T003 = FIX1 / IN_PROGRESS
```
