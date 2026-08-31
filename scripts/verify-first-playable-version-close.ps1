$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$EvidenceRoot = Join-Path $RepoRoot "docs\spec\task-evidence"

Write-Host "M32 0.1.0 First Playable Version Close Verification"
Write-Host "===================================================="

for ($Task = 1; $Task -le 12; $Task++) {
    $TaskId = "T{0:D3}" -f $Task
    $Evidence = Join-Path $EvidenceRoot "M32_0.1.0-$TaskId`_evidence.md"
    if (-not (Test-Path -LiteralPath $Evidence -PathType Leaf)) {
        throw "Missing First Playable version-close evidence: $Evidence"
    }
    Write-Host "[PASS] evidence 0.1.0-$TaskId ($(Split-Path $Evidence -Leaf))"
}

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-first-playable-bundle-c.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "First Playable Bundle C canonical chain failed with exit code $LASTEXITCODE"
}

Push-Location $RepoRoot
try {
    cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) {
        throw "cargo fmt --all -- --check failed with exit code $LASTEXITCODE"
    }

    cargo clippy --workspace --all-targets -- -D warnings
    if ($LASTEXITCODE -ne 0) {
        throw "cargo clippy --workspace --all-targets -- -D warnings failed with exit code $LASTEXITCODE"
    }

    cargo test --workspace
    if ($LASTEXITCODE -ne 0) {
        throw "cargo test --workspace failed with exit code $LASTEXITCODE"
    }

    cargo check --workspace --all-targets
    if ($LASTEXITCODE -ne 0) {
        throw "cargo check --workspace --all-targets failed with exit code $LASTEXITCODE"
    }

    git diff --check
    if ($LASTEXITCODE -ne 0) {
        throw "git diff --check failed with exit code $LASTEXITCODE"
    }

    git diff --quiet -- third_party/rustjava/jvm/src/jvm.rs
    if ($LASTEXITCODE -ne 0) {
        throw "unexpected unstaged RustJava compatibility-source change"
    }
    git diff --cached --quiet -- third_party/rustjava/jvm/src/jvm.rs
    if ($LASTEXITCODE -ne 0) {
        throw "unexpected staged RustJava compatibility-source change"
    }

    Write-Host ""
    Write-Host "Current repository scope:"
    git status --short
}
finally {
    Pop-Location
}

Write-Host ""
Write-Host "[PASS] T001-T012 complete evidence set"
Write-Host "[PASS] Bundle A/B/C canonical verifier chain"
Write-Host "[PASS] real composed desktop live display/input/audio/storage path"
Write-Host "[PASS] restart visible save/load proof"
Write-Host "[PASS] Windows manual First Playable smoke"
Write-Host "[PASS] clean shutdown/pause/product-exit/backend-fault boundary; pinned-WIE notifyDestroyed limitation documented"
Write-Host "[PASS] dependency architecture and RustJava boundary"
Write-Host "[PASS] workspace fmt/clippy/test/check + git diff quality gates"
Write-Host ""
Write-Host "M32 0.1.0 First Playable version-close verification passed."
Write-Host "0.1.0 First Playable = 12/12 DONE"
Write-Host "overall = 76/253 DONE"
Write-Host "remaining = 177"
exit 0
