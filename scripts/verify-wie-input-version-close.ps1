$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$EvidenceRoot = Join-Path $RepoRoot "docs\spec\task-evidence"

Write-Host "M32 0.0.4 Input Version Close Verification"
Write-Host "=========================================="

for ($Task = 1; $Task -le 9; $Task++) {
    $TaskId = "0.0.4-T{0:D3}" -f $Task
    $EvidenceFile = "M32_{0}_evidence.md" -f $TaskId
    $Evidence = Join-Path $EvidenceRoot $EvidenceFile

    if (-not (Test-Path -LiteralPath $Evidence -PathType Leaf)) {
        throw "Missing required Input task evidence: $Evidence"
    }

    Write-Host "[PASS] evidence $TaskId ($EvidenceFile)"
}

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-wie-input-bundle-c.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "Input Bundle C canonical functional chain failed with exit code $LASTEXITCODE"
}

Push-Location $RepoRoot
try {
    $MetadataJson = cargo metadata --format-version 1 --no-deps
    if ($LASTEXITCODE -ne 0) {
        throw "cargo metadata --format-version 1 --no-deps failed with exit code $LASTEXITCODE"
    }

    $Metadata = $MetadataJson | ConvertFrom-Json
    $InputPackage = $Metadata.packages | Where-Object { $_.name -eq "m32-input" }

    if ($null -eq $InputPackage) {
        throw "m32-input package missing from workspace metadata"
    }

    $NormalDependencies = @(
        $InputPackage.dependencies |
            Where-Object { $null -eq $_.kind -or $_.kind -eq "normal" }
    )

    if ($NormalDependencies.Count -ne 1) {
        throw "m32-input must have exactly one normal dependency; found $($NormalDependencies.Count)"
    }

    $InputDependency = $NormalDependencies[0]
    if ($InputDependency.name -ne "m32-emulator-api") {
        throw "m32-input normal dependency must be m32-emulator-api; found $($InputDependency.name)"
    }

    if ($null -ne $InputDependency.source) {
        throw "m32-input -> m32-emulator-api must remain workspace-local/path-based"
    }

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
        throw "RustJava jvm/src/jvm.rs has an uncommitted change during 0.0.4 version close"
    }

    Write-Host ""
    Write-Host "[PASS] complete T001-T009 Input evidence set"
    Write-Host "[PASS] Bundle C canonical functional chain"
    Write-Host "[PASS] Bundle B real Java Canvas input path"
    Write-Host "[PASS] exact 24-key M32 -> MIDP matrix"
    Write-Host "[PASS] 350ms / 12Hz deterministic repeat policy"
    Write-Host "[PASS] exact six-held-key capacity policy"
    Write-Host "[PASS] duplicate key-down suppression"
    Write-Host "[PASS] deterministic multi-key repeat ordering"
    Write-Host "[PASS] m32-input has exactly one normal production dependency"
    Write-Host "[PASS] m32-input -> m32-emulator-api is workspace-local/path-based"
    Write-Host "[PASS] rustfmt workspace gate"
    Write-Host "[PASS] Clippy all-targets -D warnings gate"
    Write-Host "[PASS] workspace regression suite"
    Write-Host "[PASS] workspace all-target compile gate"
    Write-Host "[PASS] git diff whitespace gate"
    Write-Host "[PASS] RustJava compatibility-source working-tree boundary"

    Write-Host ""
    Write-Host "Current repository scope:"
    git status --short

    Write-Host ""
    Write-Host "M32 0.0.4 Input version-close verification passed."
    exit 0
}
finally {
    Pop-Location
}
