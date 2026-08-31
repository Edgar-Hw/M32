# M32 0.0.6 Storage / Persistence — Bundle B Functional Stage

Status: IMPLEMENTATION / VALIDATION

```text
T006 Persistent Hosts -> WIE Platform Assembly
T007 Deterministic J2ME RMS Fixture
T008 Real Guest Save -> Session Rebuild -> Load Proof
T009 Persistent Filesystem WIE Integration / Restart Proof
```

T010 version close is attached only after T006-T009 evidence exists and this functional stage passes.

## T006 — Persistent Hosts -> WIE Platform Assembly

`m32-wie-adapter` does not take a production dependency on the concrete storage implementation.

The architecture remains:

```text
m32-storage
    -> m32-emulator-api traits

m32-wie-adapter
    -> m32-emulator-api traits
    -> WIE
```

For integration tests only, `m32-wie-adapter` has:

```toml
[dev-dependencies]
m32-storage = { path = "../m32-storage" }
```

This proves that the concrete persistent RMS repository and guest filesystem can be injected into the
existing `WiePlatformHosts` boundary without coupling production adapter code to SQLite or disk
storage.

The T006 test performs real calls through WIE's `DatabaseRepository` and `Filesystem` traits backed
by `PersistentGuestStorage`.

## T007 — Deterministic J2ME RMS Fixture

M32-owned fixture:

```text
MIDlet-Name: M32 RMS Persistence
MIDlet-1: M32 RMS Persistence,,m32.RmsPersistenceMidlet
```

Guest code uses the real Java ME RMS surface supported by the pinned WIE revision:

```text
RecordStore.openRecordStore("m32-rms", true)
RecordStore.getNumRecords()
RecordStore.addRecord(...)
RecordStore.getRecord(1)
RecordStore.closeRecordStore()
```

Fixture identity:

```text
JAD SHA-256   fa9610eec08acc1e62d0340d6c6a3d46547b9b343c83e9c204d99fc1bf129597
JAR SHA-256   a0fe3a4bffb117bee1ae9eb57924f01263738f9e7e2143daf146bcea71fdbad6
Class SHA-256 7491609e5326fe17ab8b3477a76b0bdc82903d6a79d58e02f9e528a87e3c4c19
Class major   52 / Java 8
```

The JAR is deterministic and M32-owned. No copyrighted game data is included.

## T008 — Real Guest Save -> Session Rebuild -> Load Proof

The fixture has two deterministic branches.

First session, empty storage:

```text
RecordStore count = 0
-> add payload "M32-RMS1"
-> record id must be 1
-> stdout M32_RMS_SAVED;
```

Then M32 destroys the emulator session and the `PersistentGuestStorage` instance.

A second `PersistentGuestStorage` instance is opened against the same root and a completely new WIE
J2ME session boots the exact same JAD/JAR.

Second session:

```text
RecordStore count > 0
-> getRecord(1)
-> exact byte comparison with "M32-RMS1"
-> stdout M32_RMS_LOADED_OK;
```

Pinned WIE derives the J2ME system/process ID from the JAD `MIDlet-Name`, and its RecordStore bridge
opens the platform database repository with that process ID as the application ID. Therefore the
same fixture name deterministically addresses the same M32 RMS namespace across reconstructed
sessions.

The integration test also confirms M32 storage usage for that app ID is exactly 8 payload bytes after
the first save.

## T009 — Persistent Filesystem WIE Integration / Restart Proof

T009 validates the already-existing WIE filesystem adapter against the concrete persistent disk host.

First WIE platform instance:

```text
AID  = M32 Persistent Filesystem
path = save/state.bin
write "M32-FS-PERSIST"
```

The WIE platform and storage instance are destroyed.

Second storage + WIE platform instance:

```text
exists -> true
size   -> exact payload length
read   -> exact original bytes
```

This is an adapter-level WIE persistence proof. It does not claim that every Java ME JSR-75
`FileConnection` API is implemented by the pinned runtime.

## Expected New Test Baseline

Before Bundle B:

```text
m32-wie-adapter = 68
m32-storage     = 14
workspace min   = 149
```

Bundle B adds four adapter integration tests:

```text
T006 +1
T007 +1
T008 +1
T009 +1
```

Expected:

```text
m32-wie-adapter = 72
m32-storage     = 14
workspace min   = 153
```

## Non-claims

0.0.6 does not claim:

```text
full JSR-75 FileConnection Java API coverage
save-state/rewind snapshots
cloud sync
portable cross-machine saves
per-title storage hacks
```

This version closes persistent host storage and real RMS session persistence only.
