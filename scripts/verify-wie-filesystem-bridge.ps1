$ErrorActionPreference = "Stop"

Write-Host "M32 WIE Filesystem Bridge Verification"
Write-Host "======================================"

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

$tree = cargo tree -p m32-wie-adapter --depth 1
if ($LASTEXITCODE -ne 0) {
    throw "cargo tree failed with exit code $LASTEXITCODE"
}

$asyncTraitMatch = $tree | Select-String -SimpleMatch "async-trait"
if ($null -eq $asyncTraitMatch) {
    throw "m32-wie-adapter must directly resolve async-trait for the pinned WIE Filesystem bridge."
}

Write-Host ""
Write-Host "[PASS] object-safe M32 GuestFilesystemHost contract"
Write-Host "[PASS] AID/path-preserving WIE filesystem bridge"
Write-Host "[PASS] read offset/count/buffer mapping"
Write-Host "[PASS] write/truncate mapping"
Write-Host "[PASS] host-error fallback mapping"
Write-Host "[PASS] invalid read/write count protection"
Write-Host "[PASS] direct async-trait adapter dependency"
Write-Host "[PASS] existing emulator API dependency boundary"
Write-Host "WIE filesystem bridge verification passed."
exit 0
