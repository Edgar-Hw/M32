$ErrorActionPreference = "Stop"

Write-Host "M32 WIE J2ME Session Factory Verification"
Write-Host "========================================="

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-rustjava-vendor.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "RustJava vendor verification failed with exit code $LASTEXITCODE"
}

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-rustjava-workspace-boundary.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "RustJava workspace boundary verification failed with exit code $LASTEXITCODE"
}

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-wie-platform-assembly.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "WIE Platform assembly verification failed with exit code $LASTEXITCODE"
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

$wieJ2meMatch = $tree | Select-String -SimpleMatch "wie_j2me"
if ($null -eq $wieJ2meMatch) {
    throw "m32-wie-adapter must directly resolve pinned wie_j2me."
}

Write-Host ""
Write-Host "[PASS] pinned direct wie_j2me dependency"
Write-Host "[PASS] J2MEEmulator -> WIE Emulator contract"
Write-Host "[PASS] WiePlatformAdapter ownership transfer into J2ME constructor"
Write-Host "[PASS] JAR filename/byte constructor path"
Write-Host "[PASS] Ready M32 session wrapping concrete J2ME emulator"
Write-Host "[PASS] stable M32 session-create error mapping"
Write-Host "[PASS] existing emulator API dependency boundary"
Write-Host "WIE J2ME session factory verification passed."
exit 0
