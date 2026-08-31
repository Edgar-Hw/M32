$ErrorActionPreference = "Stop"

Write-Host "M32 Tick-until-first-frame Verification"
Write-Host "======================================="

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-wie-first-frame-canvas-paint-fixture.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "T005 Canvas/Paint fixture verification failed with exit code $LASTEXITCODE"
}

cargo test -p m32-wie-adapter tests::first_frame_paint_fixture_ticks_until_guest_frame_is_captured -- --exact
if ($LASTEXITCODE -ne 0) {
    throw "T006 real tick-until-first-frame smoke failed with exit code $LASTEXITCODE"
}

Write-Host ""
Write-Host "[PASS] real PaintMidlet JAD/JAR boot"
Write-Host "[PASS] Ready -> Running session lifecycle"
Write-Host "[PASS] Display.setCurrent redraw request observation"
Write-Host "[PASS] host redraw -> wie_backend::Event::Redraw pump"
Write-Host "[PASS] pinned MIDP EventQueue repaint dispatch"
Write-Host "[PASS] guest Canvas.paint execution path"
Write-Host "[PASS] WIE screen.paint -> M32 FirstFrameCaptureDisplayHost"
Write-Host "[PASS] non-empty guest-generated first RGBA8 frame"
Write-Host "[PASS] PaintMidlet startApp stdout sentinel"
Write-Host "[PASS] previous Canvas/Paint/Capture/Positive-Boot verification chain"
Write-Host "Tick-until-first-frame verification passed."
exit 0
