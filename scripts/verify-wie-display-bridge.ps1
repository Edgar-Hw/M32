$ErrorActionPreference = "Stop"

Write-Host "M32 WIE Display Bridge Verification"
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

$tree = cargo tree -p m32-wie-adapter --depth 1
if ($LASTEXITCODE -ne 0) {
    throw "cargo tree failed with exit code $LASTEXITCODE"
}

$wieUtilMatch = $tree | Select-String -SimpleMatch "wie_util"
if ($null -eq $wieUtilMatch) {
    throw "m32-wie-adapter must directly resolve wie_util for WIE Screen error mapping."
}

Write-Host ""
Write-Host "[PASS] M32 DisplayHost contract"
Write-Host "[PASS] canonical RGBA8 frame validation"
Write-Host "[PASS] WieScreenAdapter compile bridge"
Write-Host "[PASS] synthetic WIE image -> M32 RGBA8 conversion"
Write-Host "[PASS] direct pinned wie_util adapter dependency"
Write-Host "[PASS] existing emulator API dependency boundary"
Write-Host "WIE display bridge verification passed."
exit 0
