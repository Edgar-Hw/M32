# M32 0.0.2-T001 FIX2 — tracing compatibility

## Trigger

After T001 FIX1 resolved the missing `smaf_player` package, Cargo reached WIE's tracing dependency
constraints and reported:

```text
wie_util requires tracing-attributes <0.1.29
M32 tracing 0.1.44 requires tracing-attributes ^0.1.31
```

These semver-compatible `0.1.x` requirements cannot be unified because their version ranges do not
overlap.

## Upstream baseline

The pinned WIE `Cargo.lock` uses:

```text
tracing 0.1.41
tracing-attributes 0.1.28
tracing-core 0.1.36
```

## M32 correction

```text
tracing: 0.1.44 -> 0.1.41
tracing-subscriber: remains 0.3.23
WIE revision: unchanged
SMAF revision: unchanged
```

The logging behavior/API from T009 is unchanged. This is a dependency compatibility correction
inside the active Core Adapter integration Task.

## Expected lockfile movement

On the next successful Cargo resolution, the root `Cargo.lock` should replace the Foundation
`tracing 0.1.44 / tracing-attributes 0.1.31` pair with a WIE-compatible resolution including
`tracing 0.1.41 / tracing-attributes 0.1.28`.
