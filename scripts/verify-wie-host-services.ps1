$ErrorActionPreference = "Stop"

Write-Host "M32 WIE Host Service Inventory Verification"
Write-Host "==========================================="

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
Write-Host "[PASS] M32 HostServiceKind contract"
Write-Host "[PASS] WIE host requirement inventory"
Write-Host "[PASS] pinned WIE Platform compile probe"
Write-Host "[PASS] existing emulator API dependency boundary"
Write-Host "WIE host service inventory verification passed."
exit 0
