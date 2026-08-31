$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$EvidenceRoot = Join-Path $RepoRoot "docs\spec\task-evidence"

Write-Host "M32 0.1.0 First Playable Bundle A Verification"
Write-Host "================================================"

for ($Task = 1; $Task -le 4; $Task++) {
    $TaskId = "T{0:D3}" -f $Task
    $Evidence = Join-Path $EvidenceRoot "M32_0.1.0-$TaskId`_evidence.md"
    if (-not (Test-Path -LiteralPath $Evidence -PathType Leaf)) {
        throw "Missing First Playable Bundle A evidence: $Evidence"
    }
    Write-Host "[PASS] evidence 0.1.0-$TaskId ($(Split-Path $Evidence -Leaf))"
}

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-storage-version-close.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "0.0.6 Storage version-close regression chain failed with exit code $LASTEXITCODE"
}

Push-Location $RepoRoot
try {
    $MetadataJson = cargo metadata --format-version 1 --no-deps
    if ($LASTEXITCODE -ne 0) {
        throw "cargo metadata failed with exit code $LASTEXITCODE"
    }
    $Metadata = $MetadataJson | ConvertFrom-Json

    $Desktop = $Metadata.packages | Where-Object { $_.name -eq "m32-desktop" }
    if ($null -eq $Desktop) {
        throw "m32-desktop missing from workspace metadata"
    }

    foreach ($Name in @(
        "m32-audio",
        "m32-display",
        "m32-domain",
        "m32-emulator-api",
        "m32-input",
        "m32-storage",
        "m32-ui",
        "m32-wie-adapter"
    )) {
        $Hit = @($Desktop.dependencies | Where-Object { $_.name -eq $Name })
        if ($Hit.Count -ne 1 -or $null -ne $Hit[0].source) {
            throw "m32-desktop must have one workspace-local/path dependency on $Name"
        }
    }
    Write-Host "[PASS] T001 desktop composition owns concrete M32 runtime crates"

    foreach ($Forbidden in @("wie_backend", "wie_j2me", "wie_util")) {
        $Hit = @($Desktop.dependencies | Where-Object { $_.name -eq $Forbidden })
        if ($Hit.Count -ne 0) {
            throw "desktop must not depend directly on raw WIE crate $Forbidden"
        }
    }
    Write-Host "[PASS] desktop has no raw WIE dependency"

    $Composition = Get-Content -LiteralPath (Join-Path $RepoRoot "apps\m32-desktop\src\composition.rs") -Raw
    foreach ($Token in @(
        "LiveDisplayHost",
        "SystemClockHost",
        "DesktopOutputHost",
        "DesktopExitHost",
        "DesktopVibrationHost",
        "RealtimeGuestAudioHost",
        "PersistentGuestStorage",
        "WiePlatformHosts",
        "create_j2me_jad_jar_session"
    )) {
        if ($Composition -notmatch [regex]::Escape($Token)) {
            throw "T001 composition source missing: $Token"
        }
    }
    Write-Host "[PASS] T001 concrete display/clock/output/exit/vibration/audio/storage/WIE composition"

    foreach ($Token in @(
        "--jad",
        "--jar",
        "fs::read",
        "JadRead",
        "JarRead",
        "validate_extension"
    )) {
        if ($Composition -notmatch [regex]::Escape($Token)) {
            throw "T002 local launch source missing: $Token"
        }
    }
    Write-Host "[PASS] T002 explicit local JAD+JAR path and clear file errors"

    $Api = Get-Content -LiteralPath (Join-Path $RepoRoot "crates\m32-emulator-api\src\lib.rs") -Raw
    $Adapter = Get-Content -LiteralPath (Join-Path $RepoRoot "crates\m32-wie-adapter\src\lib.rs") -Raw
    $DesktopSource = Get-Content -LiteralPath (Join-Path $RepoRoot "apps\m32-desktop\src\desktop.rs") -Raw
    $Ui = Get-Content -LiteralPath (Join-Path $RepoRoot "crates\m32-ui\src\lib.rs") -Raw

    foreach ($Token in @("fn handle_redraw(&mut self)", "latest_after", "TextureOptions::NEAREST")) {
        $All = "$Api`n$Adapter`n$Composition`n$DesktopSource"
        if ($All -notmatch [regex]::Escape($Token)) {
            throw "T003 live-frame path missing: $Token"
        }
    }
    if ($Ui -notmatch [regex]::Escape(".floor()")) {
        throw "T003 Pixel Perfect integer scale marker missing"
    }
    Write-Host "[PASS] T003 redraw seam + latest-frame replacement + NEAREST integer presentation"

    foreach ($Token in @(
        "GuestInputController",
        "repeats_due",
        "key_down",
        "key_up",
        "event.repeat"
    )) {
        $All = "$Composition`n$DesktopSource"
        if ($All -notmatch [regex]::Escape($Token)) {
            throw "T004 input pump missing: $Token"
        }
    }
    Write-Host "[PASS] T004 existing m32-input policy is wired into desktop runtime"

    cargo test -p m32-desktop composition::tests
    if ($LASTEXITCODE -ne 0) {
        throw "m32-desktop composed runtime tests failed with exit code $LASTEXITCODE"
    }

    cargo test -p m32-desktop desktop::tests::keyboard_mapping_matches_locked_first_playable_defaults
    if ($LASTEXITCODE -ne 0) {
        throw "desktop keyboard mapping test failed with exit code $LASTEXITCODE"
    }

    cargo test -p m32-wie-adapter wie_session_forwards_m32_redraw_hook_to_pinned_backend
    if ($LASTEXITCODE -ne 0) {
        throw "WIE redraw seam test failed with exit code $LASTEXITCODE"
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
Write-Host "[PASS] T001 desktop runtime composition root"
Write-Host "[PASS] T002 explicit local JAD+JAR playable launch"
Write-Host "[PASS] T003 live guest RGBA8 presentation loop"
Write-Host "[PASS] T004 desktop keyboard -> m32-input -> real guest Canvas callback"
Write-Host "[PASS] previous 0.0.6 regression chain"
Write-Host "[PASS] raw WIE/RustJava boundaries unchanged"
Write-Host "[PASS] workspace fmt/clippy/test/check quality gates"
Write-Host "[PASS] git diff whitespace gate"
Write-Host ""
Write-Host "M32 0.1.0 First Playable Bundle A verification passed."
exit 0
