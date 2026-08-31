# M32 T006 Optional Actual First-frame BMP Preview

This helper is a developer preview path for the already-running T006 guest-frame smoke.

It does not synthesize an expected screenshot.

When:

```text
M32_FIRST_FRAME_PREVIEW_BMP
```

is set, the exact `RgbaFrame` captured from:

```text
PaintMidlet
-> Canvas.paint
-> WIE screen
-> WieScreenAdapter
-> FirstFrameCaptureDisplayHost
```

is exported as a 32-bit BMP.

The exporter is test-only and uses no external image dependency.

Run:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\preview-wie-first-frame.ps1
```

Output:

```text
target\m32-preview\first-frame.bmp
```

The helper opens the resulting BMP through the Windows shell after the runtime test succeeds.

T007 remains responsible for formal exact dimension/pixel-content locking.
