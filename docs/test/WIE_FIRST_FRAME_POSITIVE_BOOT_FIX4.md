# M32 0.0.3-T003 FIX4 — robust RustJava compatibility patch application

## Trigger

FIX3 identified the actual class-loader root cause correctly, but the patch application script
reported:

```text
Expected exactly one pinned current_class_loader source block, found 0.
```

No RustJava source was modified because the safety guard rejected the match.

## Cause

The FIX3 patcher used a multiline regular expression that was unnecessarily sensitive to exact
formatting/newline representation.

The exact pinned RustJava source still contains:

```rust
async fn current_class_loader(&self) -> Result<Box<dyn ClassInstance>>
```

and the intended branch:

```rust
if let Some(x) = calling_class_class_loader {
    Ok(x)
} else {
```

## FIX4

The patcher no longer uses a whole-block multiline regex.

It now:

1. verifies the exact RustJava base revision marker;
2. verifies the `current_class_loader` function marker exists;
3. normalizes CRLF/LF in memory;
4. searches for the small exact loader-branch fragment;
5. requires exactly one occurrence;
6. inserts the RustJar -> system URLClassLoader compatibility branch;
7. verifies the patch marker and class-loader condition before writing;
8. remains idempotent when rerun.

The script still refuses to edit unexpected source.

## Scope

No change to the FIX3 compatibility behavior.

No WIE source change.

No JAD/JAR/class change.

No dependency/Cargo.lock change.

## Status

```text
0.0.3-T003 = FIX4 / IN_PROGRESS
```
