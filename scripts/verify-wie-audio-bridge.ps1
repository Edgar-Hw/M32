$ErrorActionPreference = "Stop"

Write-Host "M32 WIE Audio Bridge Verification"
Write-Host "================================="

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
Write-Host "[PASS] M32 GuestAudioHost contract"
Write-Host "[PASS] exact u32 audio handle contract"
Write-Host "[PASS] MIDI byte-preserving bridge"
Write-Host "[PASS] Wave format/sample-preserving bridge"
Write-Host "[PASS] Play/Stop command mapping"
Write-Host "[PASS] non-panicking audio host failure handling"
Write-Host "[PASS] existing emulator API dependency boundary"
Write-Host "WIE audio bridge verification passed."
exit 0
