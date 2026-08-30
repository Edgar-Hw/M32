$ErrorActionPreference = "Stop"

Write-Host "M32 Emulator Session Contract Verification"
Write-Host "=========================================="

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
Write-Host "[PASS] M32 session contract tests"
Write-Host "[PASS] WIE session adapter compile bridge"
Write-Host "[PASS] Existing emulator API dependency boundary"
Write-Host "Emulator session contract verification passed."
exit 0
