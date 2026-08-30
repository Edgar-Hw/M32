$ErrorActionPreference = "Stop"

Write-Host "M32 WIE Platform Assembly Verification"
Write-Host "======================================"

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-wie-audio-bridge.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "Audio bridge verification failed with exit code $LASTEXITCODE"
}

cargo test -p m32-wie-adapter
if ($LASTEXITCODE -ne 0) {
    throw "m32-wie-adapter tests failed with exit code $LASTEXITCODE"
}

Write-Host ""
Write-Host "[PASS] WiePlatformAdapter pinned Platform implementation"
Write-Host "[PASS] display and clock assembly"
Write-Host "[PASS] database and filesystem assembly"
Write-Host "[PASS] stdout/stderr, exit, and vibration assembly"
Write-Host "[PASS] shared M32 audio host across fresh WIE AudioSink objects"
Write-Host "[PASS] existing emulator API dependency boundary"
Write-Host "WIE platform assembly verification passed."
exit 0
