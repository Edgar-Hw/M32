$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

Write-Host 'M32 PRE77-01 Clean Reproducible Baseline Verification'
Write-Host '======================================================'

$requiredTracked = @(
    'scripts/collect-pre77-01-baseline.ps1',
    'scripts/verify-pre77-01-clean-baseline.ps1',
    'docs/test/PRE77_01_BASELINE_PROBE.md',
    'docs/spec/task-evidence/M32_PRE77-01_evidence.md'
)

$before = (& git status --porcelain=v1 --untracked-files=all | Out-String).Trim()
if ($LASTEXITCODE -ne 0) {
    throw "git status failed with exit code $LASTEXITCODE"
}
if (-not [string]::IsNullOrWhiteSpace($before)) {
    Write-Host ''
    Write-Host 'Current repository scope:'
    Write-Host $before
    throw 'PRE77-01 requires an empty Git working tree. Commit the PRE77-01 artifacts and remove accidental probe directories first.'
}
Write-Host '[PASS] repository working tree clean before verification'

foreach ($rel in $requiredTracked) {
    & git ls-files --error-unmatch -- $rel *> $null
    if ($LASTEXITCODE -ne 0) {
        throw "PRE77-01 artifact is not tracked by Git: $rel"
    }
    Write-Host "[PASS] tracked artifact $rel"
}

$collector = Join-Path $RepoRoot 'scripts\collect-pre77-01-baseline.ps1'
if (-not (Test-Path $collector)) {
    throw "Missing baseline collector: $collector"
}

powershell -ExecutionPolicy Bypass -File $collector -RequireClean
if ($LASTEXITCODE -ne 0) {
    throw "PRE77-01 clean baseline collector failed with exit code $LASTEXITCODE"
}

$after = (& git status --porcelain=v1 --untracked-files=all | Out-String).Trim()
if ($LASTEXITCODE -ne 0) {
    throw "final git status failed with exit code $LASTEXITCODE"
}
if (-not [string]::IsNullOrWhiteSpace($after)) {
    Write-Host ''
    Write-Host 'Repository became dirty during verification:'
    Write-Host $after
    throw 'PRE77-01 verifier must leave the repository clean.'
}

Write-Host ''
Write-Host '[PASS] clean committed repository baseline'
Write-Host '[PASS] exact toolchain/dependency identity captured by probe'
Write-Host '[PASS] WIE/RustJava/SMAF Cargo checkout contamination = 0'
Write-Host '[PASS] existing 0.1.0 canonical regression chain'
Write-Host '[PASS] two consecutive locked desktop builds are reproducible'
Write-Host '[PASS] verifier leaves working tree clean'
Write-Host ''
Write-Host 'PRE77-01 CLEAN REPRODUCIBLE BASELINE = PASS'
Write-Host 'Official roadmap remains 76/253 DONE; next Pre-77 gate = PRE77-02.'
