# M32 Guest Database Host Contract and WIE Bridge

Status: LOCKED BASELINE
Task: `0.0.2-T008`
Emulator API schema: `1`

## 1. 목적

T008 translates the pinned WIE database/repository APIs into M32-owned object-safe async host
contracts.

This Task defines only the Core Adapter boundary.

It does not choose SQLite tables, migrations, quota enforcement, save-backup format, or the final
database persistence layout.

## 2. Pinned WIE Database Surface

At WIE revision:

```text
f0513eb758c02736981f545ad030eed937d55f3e
```

record IDs are:

```text
RecordId = u32
```

The `database` module itself is private, while `Database`, `DatabaseRepository`, and `RecordId` are
publicly re-exported from the `wie_backend` crate root. Adapter signatures therefore use:

```text
wie_backend::RecordId
```

and never the inaccessible `wie_backend::database::RecordId` path.

Database:

```text
next_id() -> RecordId
add(data) -> RecordId
get(id) -> Option<Vec<u8>>
set(id, data) -> bool
delete(id) -> bool
get_record_ids() -> Vec<RecordId>
```

Repository:

```text
open(name, app_id) -> Box<dyn Database>
exists(name, app_id) -> bool
delete(name, app_id) -> bool
usage(app_id) -> u64
```

## 3. M32 Contracts

`m32-emulator-api` owns:

```text
GuestDatabaseRecordId = u32
GuestDatabaseError
GuestDatabaseErrorCode
GuestDatabaseHost
GuestDatabaseRepositoryHost
```

`GuestDatabaseHost: Send + Sync`.

`GuestDatabaseRepositoryHost: Send + Sync`.

The pinned WIE `Database` trait is declared `Send`, but its default `async_trait` expansion creates
`Send` futures for `&self` methods such as `next_id`, `get`, and `get_record_ids`. Therefore the
concrete adapter object must also be `Sync`, which requires the M32 database host trait object to be
`Send + Sync`.

Both use the existing boxed `HostFuture` contract so they are usable behind trait objects without
depending on WIE.

## 4. Scope Preservation

The WIE adapter preserves exactly:

```text
database name
app_id
record id
record bytes
```

The adapter does not reinterpret `name` or `app_id` as host filesystem paths.

Concrete storage isolation belongs to the later storage implementation.

## 5. WIE Error Fallback Mapping

Pinned WIE database APIs expose no separate Result error channel.

M32 host failures therefore map to:

```text
repository open error   -> unavailable no-op database
repository exists error -> false
repository delete error -> false
repository usage error  -> 0

database next_id error  -> 0
database add error      -> 0
database get error      -> None
database set error      -> false
database delete error   -> false
record_ids error        -> empty Vec
```

Every host failure produces a structured `m32::storage` warning.

The adapter logs:

```text
operation
stable error code
```

and deliberately does not log:

```text
database name
app_id
record bytes
free-form host error message
host path
```

## 6. Open Failure Database

Because WIE `DatabaseRepository::open()` cannot return an error, T008 supplies an internal
`UnavailableWieDatabase` when the M32 repository host fails to open a database.

It is intentionally non-persistent and returns only safe failure/empty values.

It is not part of the M32 public API.

## 7. Record ID Policy

M32 keeps the pinned WIE record width exactly:

```text
u32
```

T008 does not reserve, offset, remap, or reinterpret record IDs.

The fallback `0` is used only when an operation has already failed at the M32 host boundary.

A later concrete persistence implementation must preserve the behavioral expectations of the target
legacy platform before assigning durable IDs.

## 8. Storage Quota

WIE exposes:

```text
usage(app_id) -> u64
```

T008 preserves this value but does not yet enforce the locked M32 guest quota.

The concrete M32 storage implementation later applies:

```text
guest quota = 256 MiB
```

across the appropriate guest storage surfaces.

## 9. Non-Goals

T008 does not:

- create SQLite migrations;
- map database records to M32 library tables;
- persist records on disk;
- enforce quota;
- create save snapshots;
- implement audio;
- assemble the complete WIE Platform.

## 10. Acceptance

- M32 database traits remain WIE-independent;
- `RecordId` width remains exact `u32`;
- repository name/app_id are preserved;
- record IDs and bytes are preserved;
- repository and record host errors map to deterministic WIE fallbacks;
- open failure returns a safe unavailable database;
- sensitive scope/data values are not included in adapter warning fields;
- all quality gates pass.
