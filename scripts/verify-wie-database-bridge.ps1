$ErrorActionPreference = "Stop"

Write-Host "M32 WIE Database Bridge Verification"
Write-Host "===================================="

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
Write-Host "[PASS] object-safe M32 database host contracts"
Write-Host "[PASS] exact u32 database record ID contract"
Write-Host "[PASS] repository name/app_id-preserving bridge"
Write-Host "[PASS] record ID/data-preserving database bridge"
Write-Host "[PASS] repository host-error fallback mapping"
Write-Host "[PASS] record host-error fallback mapping"
Write-Host "[PASS] safe unavailable-database open fallback"
Write-Host "[PASS] existing emulator API dependency boundary"
Write-Host "WIE database bridge verification passed."
exit 0
