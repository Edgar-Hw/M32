# M32 0.0.2-T011 FIX1 — Reproduce WIE-locked RustJava revision

## Trigger

After adding the direct pinned `wie_j2me` dependency, Cargo dependency resolution failed before
compilation:

```text
error: no matching package named `java_class_proto` found
location searched: Git repository https://github.com/dlunch/RustJava.git
required by package `wie_jvm_support`
```

Consequences:

- `cargo metadata` failed;
- `cargo check` failed before compiling M32 T011 code;
- API/adapter tests could not start;
- Clippy/workspace checks failed for the same resolver reason;
- `Cargo.lock` did not update.

This was not a J2ME factory source-code failure.

## Root Cause

Pinned WIE revision:

```text
f0513eb758c02736981f545ad030eed937d55f3e
```

declares RustJava workspace dependencies using an unpinned Git URL.

However that same WIE revision's committed `Cargo.lock` records all RustJava packages at:

```text
ba5797b8eb4cf376fdd63129903d319d1d7acf98
```

At that RustJava revision, `java_class_proto` exists as package `java_class_proto`.

When M32 first made `wie_j2me` reachable as a direct adapter dependency, M32's own lockfile had no
RustJava entries yet. Cargo therefore resolved the unpinned RustJava Git dependency from the current
upstream state instead of inheriting WIE's repository lockfile.

The current upstream layout no longer yields the package required by the pinned WIE dependency graph.

## Decision

Reproduce the exact RustJava revision recorded by the pinned WIE lockfile.

M32 does not:

- change the WIE revision;
- edit WIE source;
- float RustJava to latest;
- fork WIE.

M32 vendors:

```text
https://github.com/dlunch/RustJava.git
ba5797b8eb4cf376fdd63129903d319d1d7acf98
```

under:

```text
third_party/rustjava
```

and removes the nested `.git` directory.

The upstream `LICENSE` remains included.

## Cargo Patch

The root workspace patches WIE's RustJava Git source to the vendored exact snapshot for:

```text
classfile
java_class_proto
java_constants
java_runtime
jvm
jvm_rust
```

This follows the existing T001 SMAF vendored-path-patch policy.

## Bootstrap

Run:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\vendor-rustjava.ps1
```

The script:

1. initializes a temporary Git repository;
2. fetches only the locked RustJava commit;
3. checks the resolved HEAD exactly;
4. removes `.git`;
5. writes deterministic M32 provenance/revision markers;
6. verifies required package manifests and LICENSE;
7. moves the snapshot into `third_party\rustjava`.

The operation is idempotent when an existing vendor tree has the expected revision marker.

It refuses to overwrite an unrecognized or differently pinned directory.

## Verification

Before Cargo metadata/tests, T011 verifier now runs:

```text
verify-rustjava-vendor.ps1
```

and checks:

- exact revision marker;
- all required Cargo manifests;
- `java_class_proto`;
- LICENSE;
- absence of nested `.git`.

After dependency resolution, the normal dependency tree checks must confirm `wie_j2me` is direct and
all M32/WIE boundaries remain valid.

## Lockfile

`Cargo.lock` is expected to change only after the vendored patch allows dependency resolution to
complete.

The final T011 evidence must capture the resulting lockfile/source graph.

## Status

```text
0.0.2-T011 = FIX1 / IN_PROGRESS
```
