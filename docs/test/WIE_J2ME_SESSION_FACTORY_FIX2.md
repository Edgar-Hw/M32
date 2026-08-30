# M32 0.0.2-T011 FIX2 — isolate vendored RustJava workspace

## Trigger

T011 FIX1 successfully:

- fetched RustJava exact revision `ba5797b8eb4cf376fdd63129903d319d1d7acf98`;
- restored `java_class_proto`;
- preserved all required package manifests;
- preserved LICENSE;
- removed nested `.git`.

Cargo then advanced to parsing the vendored path packages and failed with:

```text
failed to parse manifest at third_party\rustjava\classfile\Cargo.toml

error inheriting `license` from workspace root manifest's `workspace.package.license`

`workspace.package.license` was not defined
```

## Root Cause

The vendored RustJava packages use workspace inheritance.

For example:

```text
classfile/Cargo.toml
version.workspace = true
edition.workspace = true
license.workspace = true
nom = { workspace = true }
java_constants = { workspace = true }
```

The RustJava root manifest already defines its correct inheritance context:

```text
[workspace.package]
version = "0.0.1"
edition = "2024"
license = "MIT"

[workspace.dependencies]
...
```

Because the vendored path packages physically reside below the M32 workspace root and are referenced
through root `[patch]` entries, Cargo associated the path package with the outer M32 workspace during
resolution.

The M32 workspace intentionally does not duplicate RustJava's package metadata/dependency table.

Adding only `license = "MIT"` to M32 would be an incorrect partial fix because RustJava crates also
inherit many workspace dependencies.

## FIX2 Decision

Keep the vendored RustJava source as an independent nested workspace.

M32 root adds:

```toml
[workspace]
...
exclude = ["third_party/rustjava"]
```

This prevents vendored RustJava path packages from being treated as M32 workspace members.

Their own:

```text
third_party/rustjava/Cargo.toml
```

remains the authority for RustJava `workspace.package` and `workspace.dependencies` inheritance.

No upstream RustJava crate manifest is rewritten.

## Verification

`verify-rustjava-vendor.ps1` now also verifies:

- RustJava root `Cargo.toml` exists;
- `[workspace.package]` exists;
- MIT license metadata exists;
- `cargo metadata --manifest-path third_party/rustjava/Cargo.toml --no-deps` succeeds.

New:

```text
verify-rustjava-workspace-boundary.ps1
```

verifies:

- M32 workspace explicitly excludes `third_party/rustjava`;
- RustJava retains its own workspace root;
- RustJava workspace inheritance parses independently.

## Scope

No WIE revision change.

No RustJava revision change.

No J2ME factory source change.

No M32 emulator API change.

No vendored RustJava source-code edit.

No dependency version float.

## Status

```text
0.0.2-T011 = FIX2 / IN_PROGRESS
```
