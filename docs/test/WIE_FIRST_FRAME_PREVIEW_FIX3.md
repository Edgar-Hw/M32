# M32 T006 Actual First-frame Preview FIX3

## Why FIX3 exists

The original preview helper used an environment variable to request BMP export.

A user run showed:

```text
runtime test: PASS
BMP file: missing
```

and Cargo reused the existing test binary without recompiling the adapter.

The core T006 runtime result remained valid. Only the developer preview export failed.

## FIX3

Preview export no longer depends on an environment variable.

The preview script creates a temporary request marker:

```text
target/m32-preview/.request-first-frame-bmp
```

The already-existing T006 runtime test checks for that marker after it captures the real
`RgbaFrame`.

If present, the captured frame is exported to:

```text
target/m32-preview/first-frame.bmp
```

The marker is deleted immediately after the test.

The script also:

1. verifies the FIX3 source marker is really present in `m32-wie-adapter/src/lib.rs`;
2. deletes any stale BMP;
3. runs `cargo clean -p m32-wie-adapter` to force a rebuilt test binary;
4. runs the exact real T006 guest-frame test;
5. verifies the BMP exists;
6. opens the BMP through Windows.

No new test is added, so normal T006 test counts remain unchanged.

## Important

The BMP contains the actual captured M32 `RgbaFrame`.

It is not a generated expected image and not a mock screenshot.
