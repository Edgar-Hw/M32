# M32 Guest Filesystem Host Contract and WIE Bridge

Status: LOCKED BASELINE
Task: `0.0.2-T007`
Emulator API schema: `1`

## 1. 목적

T007 translates the pinned WIE AID-scoped asynchronous filesystem interface into an M32-owned,
object-safe asynchronous host contract.

This Task defines the adapter boundary only.

It does not yet map guest files onto `%LOCALAPPDATA%\M32`, enforce the final 256 MiB quota, or define
save-backup/snapshot behavior.

## 2. Pinned WIE Filesystem Contract

At WIE revision:

```text
f0513eb758c02736981f545ad030eed937d55f3e
```

the filesystem surface is AID-scoped:

```text
exists(aid, path) -> bool
size(aid, path) -> Option<usize>
read(aid, path, offset, count, buf) -> Option<usize>
write(aid, path, offset, data) -> usize
truncate(aid, path, len)
```

Every operation carries `aid`.

The M32 bridge must preserve both `aid` and guest-relative `path` exactly.

## 3. M32 Object-Safe Async Contract

`m32-emulator-api` introduces:

```text
HostFuture<'a, T>
GuestFilesystemHost
GuestFilesystemError
GuestFilesystemErrorCode
```

`GuestFilesystemHost` uses boxed `Send` futures instead of `async fn` directly so it remains usable
as:

```text
Arc<dyn GuestFilesystemHost>
```

without depending on WIE or on an external async runtime.

## 4. M32 Error Semantics

T007 starts with one stable error category:

```text
GuestFilesystemErrorCode::OperationFailed
```

This is deliberate.

Quota, invalid-path, permission, persistence, and corruption categories are not frozen into the Core
Adapter API before the concrete M32 storage implementation is designed.

The diagnostic message remains available inside M32.

The WIE adapter does not log that free-form message because a future concrete host error may contain
a host filesystem path. It logs only operation and stable error code.

## 5. WIE Fallback Mapping

The pinned WIE interface has limited/no explicit error channels.

M32 host errors therefore map as follows:

```text
exists error   -> false
size error     -> None
read error     -> None
write error    -> 0
truncate error -> no return value
```

Every mapped host failure emits a structured `m32::storage` warning.

No host absolute path, guest path, AID, or free-form host error message is logged by the adapter.

## 6. Read Contract Protection

Pinned WIE states that the caller guarantees:

```text
buf.len() >= count
```

and valid host reads satisfy:

```text
0 <= returned_count <= count
```

T007 additionally verifies a host does not report more bytes than `count` or `buf.len()`.

An invalid host count is rejected as `None` and logged as a structured warning.

## 7. Write Contract Protection

Pinned WIE requires a successful write count to equal:

```text
data.len()
```

A zero-length write is valid and therefore:

```text
data.len() == 0
written == 0
```

is accepted as success.

Any non-equal host write count is treated as an invalid/partial write:

```text
WIE return = 0
```

with a structured warning.

## 8. Async Trait Dependency

Implementing the pinned WIE async trait requires the same `async-trait` package already present in
the resolved WIE graph.

T007 makes it an exact workspace dependency:

```text
async-trait = 0.1.92
```

No WIE revision changes.

## 9. Storage Security Boundary

T007 preserves guest `aid` and `path` but does not interpret them as host paths.

A later concrete filesystem Task is responsible for:

- AID isolation;
- path normalization;
- rejecting absolute paths / traversal / drive prefixes;
- enforcing the locked guest quota;
- mapping guest files into the official M32 runtime data root.

The WIE adapter must never concatenate a guest path with a Windows host path itself.

## 10. Non-Goals

T007 does not:

- create directories under `%LOCALAPPDATA%`;
- implement actual guest persistence;
- enforce the 256 MiB quota;
- create save backups;
- implement snapshots;
- implement WIE DatabaseRepository;
- implement AudioSink;
- assemble the complete WIE Platform.

## 11. Acceptance

- `GuestFilesystemHost` is object-safe and WIE-independent;
- AID/path are preserved through the bridge;
- read offset/count/buffer semantics are preserved;
- write/truncate request values are preserved;
- host errors map to pinned WIE fallback values;
- invalid read/write counts are rejected;
- WIE filesystem types do not enter `m32-emulator-api`;
- all workspace quality gates pass.
