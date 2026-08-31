$ErrorActionPreference = "Stop"

Write-Host "M32 First-frame Timeout/Failure Boundary Verification"
Write-Host "====================================================="

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-wie-first-frame-rgba8-content.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "T007 exact RGBA8 verification failed with exit code $LASTEXITCODE"
}

cargo test -p m32-wie-adapter tests::first_frame_wait_times_out_cleanly_when_running_midlet_never_paints -- --exact
if ($LASTEXITCODE -ne 0) {
    throw "T008 healthy-no-frame timeout test failed with exit code $LASTEXITCODE"
}

cargo test -p m32-wie-adapter tests::first_frame_wait_reports_backend_fault_before_timeout -- --exact
if ($LASTEXITCODE -ne 0) {
    throw "T008 backend-fault precedence test failed with exit code $LASTEXITCODE"
}

Write-Host ""
Write-Host "[PASS] bounded healthy-session first-frame timeout"
Write-Host "[PASS] RunningMidlet reaches Running without inventing a frame"
Write-Host "[PASS] timeout diagnostics lock max_ticks/redraws/presents/state"
Write-Host "[PASS] no-frame timeout preserves first_frame=None"
Write-Host "[PASS] backend tick fault is reported before timeout"
Write-Host "[PASS] BackendTickFailed identity preserved"
Write-Host "[PASS] faulted session remains Faulted"
Write-Host "[PASS] backend failure is not mislabeled as first-frame timeout"
Write-Host "[PASS] previous exact RGBA8/real first-frame verification chain"
Write-Host "First-frame timeout/failure boundary verification passed."
exit 0
