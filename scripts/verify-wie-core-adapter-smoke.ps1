$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$Fixture = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\j2me-core-smoke-missing-main.jar"
$ExpectedSha256 = "690642fc8ce47f9d74d8898744709c219a56db98c0585289f8ebc3c24b1d0556"

Write-Host "M32 Core Adapter End-to-End Smoke Verification"
Write-Host "=============================================="

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-wie-j2me-session-factory.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "J2ME session factory verification failed with exit code $LASTEXITCODE"
}

if (-not (Test-Path -LiteralPath $Fixture -PathType Leaf)) {
    throw "Core smoke JAR fixture missing: $Fixture"
}

$ActualSha256 = (Get-FileHash -LiteralPath $Fixture -Algorithm SHA256).Hash.ToLowerInvariant()
if ($ActualSha256 -ne $ExpectedSha256) {
    throw "Core smoke JAR SHA-256 '$ActualSha256' does not match '$ExpectedSha256'."
}

cargo test -p m32-wie-adapter tests::j2me_core_smoke_ticks_real_runtime_to_stable_fault_boundary -- --exact
if ($LASTEXITCODE -ne 0) {
    throw "J2ME core tick smoke failed with exit code $LASTEXITCODE"
}

Write-Host ""
Write-Host "[PASS] deterministic synthetic JAR fixture"
Write-Host "[PASS] locked synthetic JAR SHA-256"
Write-Host "[PASS] real pinned J2ME runtime tick path"
Write-Host "[PASS] bounded startup failure smoke"
Write-Host "[PASS] upstream WIE tick panic containment"
Write-Host "[PASS] stable BackendTickFailed mapping"
Write-Host "[PASS] Ready -> Faulted lifecycle transition"
Write-Host "[PASS] previous Core Adapter verification chain"
Write-Host "Core Adapter end-to-end smoke verification passed."
exit 0
