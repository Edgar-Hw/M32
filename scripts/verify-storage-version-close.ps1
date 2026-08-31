$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$EvidenceRoot = Join-Path $RepoRoot "docs\spec\task-evidence"

Write-Host "M32 0.0.6 Storage Version Close Verification"
Write-Host "============================================="

for ($Task = 1; $Task -le 9; $Task++) {
    $TaskId = "T{0:D3}" -f $Task
    $Evidence = Join-Path $EvidenceRoot "M32_0.0.6-$TaskId`_evidence.md"

    if (-not (Test-Path -LiteralPath $Evidence -PathType Leaf)) {
        throw "Missing Storage evidence: $Evidence"
    }

    Write-Host "[PASS] evidence 0.0.6-$TaskId ($(Split-Path $Evidence -Leaf))"
}

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-storage-bundle-b.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "Storage Bundle B canonical functional chain failed with exit code $LASTEXITCODE"
}

Push-Location $RepoRoot
try {
    $MetadataJson = cargo metadata --format-version 1 --no-deps
    if ($LASTEXITCODE -ne 0) {
        throw "cargo metadata failed with exit code $LASTEXITCODE"
    }

    $Metadata = $MetadataJson | ConvertFrom-Json

    $Storage = $Metadata.packages | Where-Object { $_.name -eq "m32-storage" }
    if ($null -eq $Storage) {
        throw "m32-storage missing from workspace metadata"
    }

    $ApiDependency = @(
        $Storage.dependencies |
            Where-Object { $_.name -eq "m32-emulator-api" }
    )
    if ($ApiDependency.Count -ne 1) {
        throw "m32-storage must have exactly one m32-emulator-api dependency"
    }
    if ($null -ne $ApiDependency[0].source) {
        throw "m32-storage -> m32-emulator-api must remain workspace-local/path-based"
    }

    $Rusqlite = @(
        $Storage.dependencies |
            Where-Object { $_.name -eq "rusqlite" }
    )
    if ($Rusqlite.Count -ne 1) {
        throw "m32-storage must have exactly one rusqlite dependency"
    }
    if ($Rusqlite[0].req -ne "=0.37.0") {
        throw "rusqlite must remain pinned exactly to =0.37.0; found $($Rusqlite[0].req)"
    }

    $Adapter = $Metadata.packages | Where-Object { $_.name -eq "m32-wie-adapter" }
    if ($null -eq $Adapter) {
        throw "m32-wie-adapter missing from workspace metadata"
    }

    $AdapterStorage = @(
        $Adapter.dependencies |
            Where-Object { $_.name -eq "m32-storage" }
    )
    if ($AdapterStorage.Count -ne 1) {
        throw "m32-wie-adapter must have exactly one m32-storage dependency entry"
    }
    if ($AdapterStorage[0].kind -ne "dev") {
        throw "m32-wie-adapter -> m32-storage must remain dev-only"
    }
    if ($null -ne $AdapterStorage[0].source) {
        throw "m32-wie-adapter -> m32-storage dev dependency must remain workspace-local/path-based"
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
Write-Host "[PASS] complete T001-T009 Storage evidence set"
Write-Host "[PASS] Bundle B canonical functional chain"
Write-Host "[PASS] persistent hosts assemble through WIE platform"
Write-Host "[PASS] deterministic Java ME RecordStore fixture"
Write-Host "[PASS] real guest RMS save -> emulator/storage rebuild -> load"
Write-Host "[PASS] WIE filesystem restart persistence"
Write-Host "[PASS] SQLite WAL / FK / 2000ms / schema-v1 policy"
Write-Host "[PASS] app_id/AID isolation and traversal boundary"
Write-Host "[PASS] exact rusqlite 0.37.0 pin"
Write-Host "[PASS] m32-storage -> m32-emulator-api dependency boundary"
Write-Host "[PASS] m32-wie-adapter -> m32-storage remains dev-only"
Write-Host "[PASS] rustfmt workspace gate"
Write-Host "[PASS] Clippy all-targets -D warnings gate"
Write-Host "[PASS] workspace regression suite"
Write-Host "[PASS] workspace all-target compile gate"
Write-Host "[PASS] git diff whitespace gate"
Write-Host "[PASS] RustJava compatibility-source staged/unstaged boundary"
Write-Host ""
Write-Host "M32 0.0.6 Storage version-close verification passed."
exit 0
