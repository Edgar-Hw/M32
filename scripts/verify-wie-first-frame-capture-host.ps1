$ErrorActionPreference = "Stop"

Write-Host "M32 First-frame Capture Host Verification"
Write-Host "========================================="

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-wie-first-frame-positive-boot.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "T003 positive boot verification failed with exit code $LASTEXITCODE"
}

cargo test -p m32-wie-adapter tests::first_frame_capture_host_does_not_invent_frame_before_present -- --exact
if ($LASTEXITCODE -ne 0) {
    throw "First-frame empty-state test failed with exit code $LASTEXITCODE"
}

cargo test -p m32-wie-adapter tests::first_frame_capture_host_locks_the_first_presented_frame -- --exact
if ($LASTEXITCODE -ne 0) {
    throw "First-frame lock test failed with exit code $LASTEXITCODE"
}

cargo test -p m32-wie-adapter tests::wie_screen_adapter_feeds_rgba8_into_first_frame_capture_host -- --exact
if ($LASTEXITCODE -ne 0) {
    throw "WIE screen -> capture host integration test failed with exit code $LASTEXITCODE"
}

Write-Host ""
Write-Host "[PASS] resize/redraw cannot fake first frame"
Write-Host "[PASS] first presented RGBA8 frame capture"
Write-Host "[PASS] first frame remains immutable after later presents"
Write-Host "[PASS] presentation count observation"
Write-Host "[PASS] WIE Screen -> M32 RGBA8 -> capture host path"
Write-Host "[PASS] injectable capture display test platform"
Write-Host "[PASS] previous positive-boot/Core Adapter verification chain"
Write-Host "First-frame capture host verification passed."
exit 0
