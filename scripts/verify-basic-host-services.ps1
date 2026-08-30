$ErrorActionPreference = "Stop"

Write-Host "M32 Basic Host Service Verification"
Write-Host "==================================="

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-emulator-api-boundary.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "Emulator API boundary verification failed with exit code $LASTEXITCODE"
}

cargo test -p m32-emulator-api
if ($LASTEXITCODE -ne 0) {
    throw "m32-emulator-api tests failed with exit code $LASTEXITCODE"
}

cargo test -p m32-wie-adapter
if ($LASTEXITCODE -ne 0) {
    throw "m32-wie-adapter tests failed with exit code $LASTEXITCODE"
}

Write-Host ""
Write-Host "[PASS] ClockHost epoch-millisecond contract"
Write-Host "[PASS] raw guest stdout/stderr contract"
Write-Host "[PASS] guest exit request contract"
Write-Host "[PASS] vibration request contract"
Write-Host "[PASS] WIE basic host bridge"
Write-Host "[PASS] existing emulator API dependency boundary"
Write-Host "Basic host service verification passed."
exit 0
