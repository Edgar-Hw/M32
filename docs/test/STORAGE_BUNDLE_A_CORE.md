# M32 0.0.6 Storage / Persistence — Accelerated Bundle A

Status: IMPLEMENTATION BASELINE

```text
Bundle A: T001-T005
Bundle B: T006-T010
```

## Locked 0.0.6 Task Sequence

```text
T001 Persistent Storage Root / SQLite Policy
T002 Persistent RMS Database Repository
T003 Persistent Guest Filesystem
T004 Per-game Isolation / Traversal Boundary
T005 Restart Persistence / Usage Accounting

T006 Persistent Hosts -> WIE Platform Assembly
T007 Deterministic J2ME RMS Fixture
T008 Real Guest Save -> Session Rebuild -> Load Proof
T009 Persistent Filesystem WIE Integration / Restart Proof
T010 Storage Canonical Integration / Version Close
```

These names are the working sequence locked for the current implementation.

## T001

M32's already-locked Windows runtime root remains:

```text
%LOCALAPPDATA%\M32
```

Storage adds beneath that root:

```text
storage.sqlite3
guest-files\
```

SQLite baseline:

```text
schema version = 1
journal_mode   = WAL
foreign_keys   = ON
busy_timeout   = 2000ms
```

`rusqlite` is pinned exactly:

```text
rusqlite = 0.37.0
features = bundled
```

The bundled SQLite feature is deliberate for the Windows distribution boundary.

## T002

Persistent RMS/database state uses SQLite tables:

```text
guest_databases
guest_records
```

Isolation key:

```text
(app_id, database name)
```

Record IDs:

```text
start at 1
monotonically advance
deleted IDs are not reused
record IDs are returned sorted ascending
```

Repository operations:

```text
open
exists
delete
usage
```

Database operations:

```text
next_id
add
get
set
delete
record_ids
```

`usage(app_id)` counts persistent record payload bytes owned by that application.

## T003

Guest filesystem bytes are stored below:

```text
%LOCALAPPDATA%\M32\guest-files
```

Semantics:

```text
exists
size
read
write
truncate
create-on-write
create-on-truncate
write-past-EOF zero fill
truncate growth zero fill
write/truncate sync_data durability boundary
EOF read -> Some(0)
missing file read -> None
```

## T004

WIE filesystem operations are AID-scoped. M32 preserves that as a hard isolation boundary.

AID and path components are encoded into safe host path components rather than concatenated as raw
Windows paths.

Rejected guest paths include:

```text
.
..
NUL
path that resolves to no file component
```

This prevents parent traversal and Windows absolute-path escape.

Database isolation is separately keyed by exact `app_id`.

No title/hash-specific behavior exists.

## T005

The Bundle A restart contract requires both persistence surfaces to survive destruction and
reconstruction of `PersistentGuestStorage` against the same M32 root.

It also proves:

```text
RMS bytes survive reopen
RMS next_id survives reopen
guest file bytes survive reopen
different app_id / AID cannot observe another game's state
usage remains app-scoped
```

## Expected Test Baseline

Prior Audio-close minimum:

```text
135
```

Bundle A adds:

```text
m32-storage = 14 tests
```

Expected minimum workspace total:

```text
149 tests
```

## Bundle B

Bundle B will not invent a second persistence model.

It will inject these exact hosts into the existing WIE platform bridge and prove real guest
persistence across reconstructed emulator sessions.
