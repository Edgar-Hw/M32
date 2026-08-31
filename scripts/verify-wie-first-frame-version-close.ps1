$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

Write-Host "M32 0.0.3 First Frame Version Close Verification"
Write-Host "================================================"

$EvidenceRoot = Join-Path $RepoRoot "docs\spec\task-evidence"
for ($Task = 1; $Task -le 9; $Task++) {
    $TaskId = "0.0.3-T{0:D3}" -f $Task
    $EvidenceFile = "M32_{0}_evidence.md" -f $TaskId
    $Evidence = Join-Path $EvidenceRoot $EvidenceFile

    if (-not (Test-Path -LiteralPath $Evidence -PathType Leaf)) {
        throw "Missing required First Frame task evidence: $Evidence"
    }

    Write-Host "[PASS] evidence $TaskId ($EvidenceFile)"
}

$IntegrationVerifier = Join-Path $PSScriptRoot "verify-wie-first-frame-integration.ps1"
if (-not (Test-Path -LiteralPath $IntegrationVerifier -PathType Leaf)) {
    throw "Canonical First Frame integration verifier missing: $IntegrationVerifier"
}

powershell -ExecutionPolicy Bypass -File $IntegrationVerifier
if ($LASTEXITCODE -ne 0) {
    throw "Canonical First Frame integration verifier failed with exit code $LASTEXITCODE"
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

    git diff --quiet -- Cargo.lock
    if ($LASTEXITCODE -ne 0) {
        throw "Cargo.lock has an uncommitted change during 0.0.3 version close."
    }

    git diff --quiet -- third_party/rustjava/jvm/src/jvm.rs
    if ($LASTEXITCODE -ne 0) {
        throw "RustJava jvm/src/jvm.rs has an uncommitted change during 0.0.3 version close."
    }

    Write-Host ""
    Write-Host "[PASS] complete T001-T009 task evidence set"
    Write-Host "[PASS] canonical First Frame integration verifier"
    Write-Host "[PASS] SUCCESS / TIMEOUT / FAULT First Frame contract"
    Write-Host "[PASS] rustfmt workspace gate"
    Write-Host "[PASS] Clippy workspace all-targets -D warnings gate"
    Write-Host "[PASS] workspace unit/integration regression suite"
    Write-Host "[PASS] workspace all-target compile gate"
    Write-Host "[PASS] git diff whitespace gate"
    Write-Host "[PASS] Cargo.lock working-tree boundary"
    Write-Host "[PASS] RustJava compatibility-source working-tree boundary"
    Write-Host ""
    Write-Host "Current repository scope:"
    git status --short

    Write-Host ""
    Write-Host "M32 0.0.3 First Frame version-close verification passed."
    exit 0
}
finally {
    Pop-Location
}
