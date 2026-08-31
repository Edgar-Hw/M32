# M32 0.0.6 Storage / Persistence — T010 Version Close

Task:

```text
0.0.6-T010 Storage Canonical Integration / Version Close
```

T010 adds no new guest-visible storage feature.

It closes the complete T001-T009 storage implementation after the evidence and regression gates pass.

## Closed Storage Contract

```text
%LOCALAPPDATA%\M32 storage root
storage.sqlite3
guest-files\

SQLite schema version 1
WAL
foreign_keys ON
busy_timeout 2000ms

persistent RMS database repository
monotonic record IDs
app_id database isolation
persistent guest filesystem
AID filesystem isolation
parent traversal rejection
restart persistence

persistent hosts injected into WIE platform
real Java ME RecordStore fixture
real guest save -> emulator rebuild -> load
WIE filesystem platform rebuild persistence
```

## Dependency Boundary

Core storage:

```text
m32-storage -> m32-emulator-api
rusqlite = =0.37.0 with bundled SQLite
```

WIE adapter:

```text
m32-storage is dev-only
no production SQLite dependency in m32-wie-adapter
```

## Expected Final Minimum Test Baseline

Prior Storage Bundle A minimum:

```text
149
```

Bundle B adds four adapter tests:

```text
+4
```

Expected minimum:

```text
153 tests
```

Critical package baselines:

```text
m32-storage      14
m32-wie-adapter  72
m32-audio        15
m32-input         6
m32-emulator-api 32
```

## Non-claims

0.0.6 does not claim:

```text
complete Java JSR-75 FileConnection coverage
cloud synchronization
portable save-state snapshots
cross-device save migration
database encryption
per-title storage hacks
```

These are later product/compatibility scope.

## Next Version

```text
0.1.0 First Playable
12 tasks
```
