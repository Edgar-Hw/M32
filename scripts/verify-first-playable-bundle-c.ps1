$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$EvidenceRoot = Join-Path $RepoRoot "docs\spec\task-evidence"
$FixtureRoot = Join-Path $RepoRoot "apps\m32-desktop\test-fixtures"

Write-Host "M32 0.1.0 First Playable Bundle C Verification"
Write-Host "================================================"

for ($Task = 9; $Task -le 11; $Task++) {
    $TaskId = "T{0:D3}" -f $Task
    $Evidence = Join-Path $EvidenceRoot "M32_0.1.0-$TaskId`_evidence.md"
    if (-not (Test-Path -LiteralPath $Evidence -PathType Leaf)) {
        throw "Missing First Playable Bundle C evidence: $Evidence"
    }
    Write-Host "[PASS] evidence 0.1.0-$TaskId ($(Split-Path $Evidence -Leaf))"
}

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-first-playable-bundle-b.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "Previous First Playable Bundle B canonical chain failed with exit code $LASTEXITCODE"
}

Push-Location $RepoRoot
try {
    $Composition = Get-Content -LiteralPath "apps\m32-desktop\src\composition.rs" -Raw
    $Desktop = Get-Content -LiteralPath "apps\m32-desktop\src\desktop.rs" -Raw
    $FixtureSource = Get-Content -LiteralPath "apps\m32-desktop\test-fixtures\src\m32\FirstPlayableMidlet.java" -Raw
    $ManualEvidence = Get-Content -LiteralPath "docs\spec\task-evidence\M32_0.1.0-T010_evidence.md" -Raw

    foreach ($Token in @(
        "first_playable_restart_restores_visible_saved_value_through_full_composition",
        "M32_FP_RUNNING:1;",
        "assert_eq!(restored.frame.pixels, saved_pixels)"
    )) {
        if ($Composition -notmatch [regex]::Escape($Token)) {
            throw "T009 restart visible save/load token missing: $Token"
        }
    }
    cargo test -p m32-desktop composition::tests::first_playable_restart_restores_visible_saved_value_through_full_composition
    if ($LASTEXITCODE -ne 0) {
        throw "T009 composed restart visible save/load test failed with exit code $LASTEXITCODE"
    }
    Write-Host "[PASS] T009 complete product composition restart restores exact visible saved frame"

    if ($ManualEvidence -notmatch [regex]::Escape("MANUAL_WINDOWS_FIRST_PLAYABLE_SMOKE: PASS")) {
        throw "T010 manual Windows First Playable smoke is still pending. Run scripts\run-first-playable-manual-smoke.ps1"
    }
    Write-Host "[PASS] T010 Windows manual First Playable smoke evidence"

    foreach ($Token in @(
        "set_paused",
        "playable_runtime_drop",
        "product_exit_request_reaches_runtime_without_panic",
        "backend_fault_remains_an_error_at_product_boundary"
    )) {
        if ($Composition -notmatch [regex]::Escape($Token)) {
            throw "T011 composition lifecycle token missing: $Token"
        }
    }
    foreach ($Token in @(
        'WindowEvent::CloseRequested',
        'WindowEvent::Focused(focused)',
        'stop_playable("backend_exit")',
        'stop_playable("window_close")',
        'stop_playable("fault")'
    )) {
        if ($Desktop -notmatch [regex]::Escape($Token)) {
            throw "T011 desktop lifecycle token missing: $Token"
        }
    }
    if ($FixtureSource -notmatch [regex]::Escape("0x4D, 0x4D, 0x4D, 0x44") -or
        $FixtureSource -notmatch [regex]::Escape("(byte) 0x90")) {
        throw "T010 canonical First Playable fixture must contain the M32-generated audible SMAF note"
    }

    cargo test -p m32-desktop composition::tests::playable_pause_resume_stops_pump_without_destroying_runtime
    if ($LASTEXITCODE -ne 0) {
        throw "T011 pause/resume product test failed with exit code $LASTEXITCODE"
    }
    cargo test -p m32-desktop composition::tests::product_exit_request_reaches_runtime_without_panic
    if ($LASTEXITCODE -ne 0) {
        throw "T011 product exit-host signal test failed with exit code $LASTEXITCODE"
    }
    cargo test -p m32-wie-adapter tests::wie_platform_delegates_output_exit_and_vibration
    if ($LASTEXITCODE -ne 0) {
        throw "T011 WIE Platform::exit delegation test failed with exit code $LASTEXITCODE"
    }
    cargo test -p m32-desktop composition::tests::backend_fault_remains_an_error_at_product_boundary
    if ($LASTEXITCODE -ne 0) {
        throw "T011 backend-fault product test failed with exit code $LASTEXITCODE"
    }

    cargo check -p m32-desktop --target x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) {
        throw "T011 Windows desktop lifecycle compile gate failed with exit code $LASTEXITCODE"
    }

    git diff --check
    if ($LASTEXITCODE -ne 0) {
        throw "git diff --check failed with exit code $LASTEXITCODE"
    }
}
finally {
    Pop-Location
}

Write-Host ""
Write-Host "[PASS] T009 restart save/load visible-state proof"
Write-Host "[PASS] T010 Windows manual playable smoke"
Write-Host "[PASS] T011 shutdown/pause/product-exit/fault/storage lifetime boundary"
Write-Host "[PASS] previous Bundle B -> Bundle A canonical chain"
Write-Host ""
Write-Host "M32 0.1.0 First Playable Bundle C verification passed."
exit 0
