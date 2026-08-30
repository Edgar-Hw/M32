# M32 0.0.2-T008 FIX1 — WIE RecordId visibility and async Send/Sync compatibility

## Trigger

T008 was correctly applied and the M32 API tests passed:

```text
m32-emulator-api: 22 passed
```

The WIE adapter compile failed with two independent issues.

### 1. Private module path

Incorrect adapter signature:

```text
wie_backend::database::RecordId
```

The pinned WIE crate declares:

```text
mod database;
```

so the module path is private.

However WIE publicly re-exports the alias at crate root:

```text
pub use database::{Database, DatabaseRepository, RecordId};
```

FIX1 therefore uses:

```text
wie_backend::RecordId
```

The underlying locked type remains:

```text
u32
```

### 2. Async Send future requires Sync receiver

The pinned WIE database source uses default `async_trait`:

```text
#[async_trait::async_trait]
pub trait Database: Send
```

Methods such as:

```text
next_id(&self)
get(&self, ...)
get_record_ids(&self)
```

expand to `Send` futures.

The adapter stores:

```text
Box<dyn GuestDatabaseHost>
```

and those `&self` async calls therefore require the trait object to be `Sync`.

FIX1 changes the M32 contract from:

```text
GuestDatabaseHost: Send
```

to:

```text
GuestDatabaseHost: Send + Sync
```

This matches the effective threading requirement imposed by the pinned WIE async interface.

## Scope

No record semantics change.

No repository fallback mapping change.

No persistence behavior change.

No dependency version change.

No WIE revision change.

No database name/app_id logging change.
