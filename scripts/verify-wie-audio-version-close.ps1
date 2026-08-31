$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$EvidenceRoot = Join-Path $RepoRoot "docs\spec\task-evidence"

Write-Host "M32 0.0.5 Audio Version Close Verification"
Write-Host "=========================================="

for ($Task = 1; $Task -le 9; $Task++) {
    $TaskId = "T{0:D3}" -f $Task
    $Evidence = Join-Path $EvidenceRoot "M32_0.0.5-$TaskId`_evidence.md"

    if (-not (Test-Path -LiteralPath $Evidence -PathType Leaf)) {
        throw "Missing Audio evidence: $Evidence"
    }

    Write-Host "[PASS] evidence 0.0.5-$TaskId ($(Split-Path $Evidence -Leaf))"
}

$T009Evidence = Join-Path $EvidenceRoot "M32_0.0.5-T009_evidence.md"
$T009Text = [System.IO.File]::ReadAllText($T009Evidence)
if (-not $T009Text.Contains("MANUAL_AUDIBLE_SMOKE: PASS")) {
    throw "T009 evidence does not contain MANUAL_AUDIBLE_SMOKE: PASS"
}

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-wie-audio-bundle-b.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "Audio Bundle B canonical functional chain failed with exit code $LASTEXITCODE"
}

Push-Location $RepoRoot
try {
    $MetadataJson = cargo metadata --format-version 1 --no-deps
    if ($LASTEXITCODE -ne 0) {
        throw "cargo metadata failed with exit code $LASTEXITCODE"
    }

    $Metadata = $MetadataJson | ConvertFrom-Json
    $AudioPackage = $Metadata.packages | Where-Object { $_.name -eq "m32-audio" }
    if ($null -eq $AudioPackage) {
        throw "m32-audio missing from workspace metadata"
    }

    $UnconditionalNormalDependencies = @(
        $AudioPackage.dependencies |
            Where-Object {
                ($null -eq $_.kind -or $_.kind -eq "normal") -and
                $null -eq $_.target
            }
    )

    if ($UnconditionalNormalDependencies.Count -ne 1) {
        throw "m32-audio must have exactly one unconditional normal dependency; found $($UnconditionalNormalDependencies.Count)"
    }

    $ApiDependency = $UnconditionalNormalDependencies[0]
    if ($ApiDependency.name -ne "m32-emulator-api") {
        throw "m32-audio unconditional dependency must be m32-emulator-api"
    }
    if ($null -ne $ApiDependency.source) {
        throw "m32-audio -> m32-emulator-api must remain workspace-local/path-based"
    }

    $CpalDependencies = @(
        $AudioPackage.dependencies |
            Where-Object { $_.name -eq "cpal" }
    )

    if ($CpalDependencies.Count -ne 1) {
        throw "m32-audio must have exactly one CPAL dependency; found $($CpalDependencies.Count)"
    }

    $Cpal = $CpalDependencies[0]
    if ($Cpal.req -ne "=0.18.2") {
        throw "CPAL must remain pinned exactly to =0.18.2; found $($Cpal.req)"
    }
    if ($null -eq $Cpal.target) {
        throw "CPAL must remain target-specific rather than unconditional"
    }
    if (-not ($Cpal.target -like "*windows*")) {
        throw "CPAL target expression must remain Windows-specific; found $($Cpal.target)"
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
Write-Host "[PASS] complete T001-T009 Audio evidence set"
Write-Host "[PASS] T009 physical audible Windows smoke evidence"
Write-Host "[PASS] Bundle B canonical functional chain"
Write-Host "[PASS] real Java MMAPI -> WIE -> M32 audio path"
Write-Host "[PASS] 48kHz f32 stereo canonical pipeline"
Write-Host "[PASS] deterministic Wave scheduling / Stop / repeat"
Write-Host "[PASS] baseline MIDI NoteOn/NoteOff renderer"
Write-Host "[PASS] exact 80ms / 3840-frame pause fade"
Write-Host "[PASS] Windows CPAL output boundary"
Write-Host "[PASS] exact CPAL 0.18.2 pin"
Write-Host "[PASS] m32-audio unconditional core dependency boundary"
Write-Host "[PASS] rustfmt workspace gate"
Write-Host "[PASS] Clippy all-targets -D warnings gate"
Write-Host "[PASS] workspace regression suite"
Write-Host "[PASS] workspace all-target compile gate"
Write-Host "[PASS] git diff whitespace gate"
Write-Host "[PASS] RustJava compatibility-source staged/unstaged boundary"
Write-Host ""
Write-Host "M32 0.0.5 Audio version-close verification passed."
exit 0
