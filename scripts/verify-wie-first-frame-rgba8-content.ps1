$ErrorActionPreference = "Stop"

Write-Host "M32 First-frame RGBA8 Content Verification"
Write-Host "=========================================="

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-wie-first-frame-tick-harness.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "T006 tick-until-first-frame verification failed with exit code $LASTEXITCODE"
}

cargo test -p m32-wie-adapter tests::first_frame_paint_fixture_locks_exact_rgba8_dimensions_and_content -- --exact
if ($LASTEXITCODE -ne 0) {
    throw "T007 exact RGBA8 content test failed with exit code $LASTEXITCODE"
}

Write-Host ""
Write-Host "[PASS] exact 176x220 first-frame dimensions"
Write-Host "[PASS] exact 154880-byte RGBA8 framebuffer"
Write-Host "[PASS] exact 38720-pixel frame"
Write-Host "[PASS] exact 16x16 M32 RED marker region"
Write-Host "[PASS] exact 256 RED pixel count"
Write-Host "[PASS] exact M32 BG0 background region"
Write-Host "[PASS] exact 38464 BG0 pixel count"
Write-Host "[PASS] all alpha bytes locked to 255 through exact pixel values"
Write-Host "[PASS] every framebuffer coordinate validated"
Write-Host "[PASS] first captured frame remains the locked frame"
Write-Host "[PASS] previous real first-frame verification chain"
Write-Host "First-frame RGBA8 content verification passed."
exit 0
