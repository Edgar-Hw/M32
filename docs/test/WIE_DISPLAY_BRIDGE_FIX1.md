# M32 0.0.2-T005 FIX1 — verifier array matching

## Trigger

The T005 implementation itself passed:

- `m32-emulator-api`: 12 tests
- `m32-wie-adapter`: 11 tests
- direct `wie_util` shown by `cargo tree`
- Clippy: exit 0
- workspace tests: exit 0
- all-target check: exit 0
- git diff check: exit 0

However `scripts/verify-wie-display-bridge.ps1` incorrectly failed the `wie_util` check.

## Root cause

PowerShell comparison:

```powershell
if ($tree -notmatch "wie_util")
```

was applied to the multi-line array returned by `cargo tree`.

For an array, `-notmatch` returns every element that does not match. Because most tree lines do not
contain `wie_util`, the returned array is non-empty and the `if` condition becomes true even when
one line correctly contains `wie_util`.

## Correction

FIX1 changes the check to:

```powershell
$wieUtilMatch = $tree | Select-String -SimpleMatch "wie_util"
if ($null -eq $wieUtilMatch) {
    throw ...
}
```

This tests existence of at least one matching dependency-tree line.

No Rust source, Cargo dependency, WIE revision, or runtime behavior changes in FIX1.
