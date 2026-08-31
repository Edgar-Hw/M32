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

# The immediately previous release gate must still pass after the desktop changes.
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
    $Ui = $Metadata.packages | Where-Object { $_.name -eq "m32-ui" }
    $Display = $Metadata.packages | Where-Object { $_.name -eq "m32-display" }

    if ($null -eq $Desktop -or $null -eq $Ui -or $null -eq $Display) {
        throw "m32-desktop/m32-ui/m32-display must all exist in workspace metadata"
    }

    foreach ($Forbidden in @("m32-wie-adapter", "m32-storage", "m32-audio")) {
        $Hit = @($Desktop.dependencies | Where-Object { $_.name -eq $Forbidden })
        if ($Hit.Count -ne 0) {
            throw "Bundle A boundary violation: m32-desktop must not directly depend on $Forbidden"
        }
    }
    Write-Host "[PASS] desktop composition boundary excludes WIE/storage/audio direct dependencies"

    $ExpectedDesktopLocal = @("m32-display", "m32-domain", "m32-ui")
    foreach ($Name in $ExpectedDesktopLocal) {
        $Hit = @($Desktop.dependencies | Where-Object { $_.name -eq $Name })
        if ($Hit.Count -ne 1 -or $null -ne $Hit[0].source) {
            throw "m32-desktop must have one workspace-local/path dependency on $Name"
        }
    }
    Write-Host "[PASS] desktop -> domain/ui/display workspace-local composition boundary"

    $ExpectedPins = @{
        "egui" = "=0.36.1"
        "egui-wgpu" = "=0.36.1"
        "egui-winit" = "=0.36.1"
        "pollster" = "=0.4.0"
        "wgpu" = "=30.0.1"
        "winit" = "=0.30.13"
    }

    foreach ($Pair in $ExpectedPins.GetEnumerator()) {
        $Found = @(
            $Metadata.packages.dependencies |
            Where-Object { $_.name -eq $Pair.Key }
        )
        if ($Found.Count -eq 0) {
            throw "Dependency $($Pair.Key) not present in workspace package metadata"
        }
        foreach ($Dependency in $Found) {
            if ($Dependency.req -ne $Pair.Value) {
                throw "$($Pair.Key) must remain pinned exactly to $($Pair.Value); found $($Dependency.req)"
            }
        }
    }
    Write-Host "[PASS] exact egui/winit/wgpu/pollster Bundle A dependency pins"

    $RootCargo = Get-Content -LiteralPath (Join-Path $RepoRoot "Cargo.toml") -Raw
    $LockedWie = "f0513eb758c02736981f545ad030eed937d55f3e"
    if (($RootCargo | Select-String -Pattern $LockedWie -AllMatches).Matches.Count -lt 4) {
        throw "Locked WIE revision is missing from expected root Cargo.toml dependency/metadata positions"
    }
    Write-Host "[PASS] locked WIE revision unchanged"

    $DesktopSource = Get-Content -LiteralPath (Join-Path $RepoRoot "apps\m32-desktop\src\desktop.rs") -Raw
    foreach ($Token in @(
        "REFERENCE_WINDOW_WIDTH",
        "REFERENCE_WINDOW_HEIGHT",
        "MIN_WINDOW_WIDTH",
        "MIN_WINDOW_HEIGHT",
        "ApplicationHandler",
        "DisplayRenderer::new",
        "gpu_device_lost",
        "gpu_renderer_recovered"
    )) {
        if ($DesktopSource -notmatch [regex]::Escape($Token)) {
            throw "Desktop source missing locked Bundle A token: $Token"
        }
    }
    Write-Host "[PASS] T001 native window and T002 renderer-recovery composition tokens"

    $UiSource = Get-Content -LiteralPath (Join-Path $RepoRoot "crates\m32-ui\src\lib.rs") -Raw
    foreach ($Token in @(
        "BOOT_DURATION_MS: u64 = 1_000",
        "PLAY_VIEWPORT_WIDTH: f32 = 1_040.0",
        "PLAY_GAP: f32 = 16.0",
        "PLAY_SIDE_DECK_WIDTH: f32 = 264.0",
        "MEMORY",
        "SOUND",
        "GAME CARD",
        "Waiting for local JAD/JAR"
    )) {
        if ($UiSource -notmatch [regex]::Escape($Token)) {
            throw "UI source missing locked Bundle A token: $Token"
        }
    }
    Write-Host "[PASS] T003 one-second boot and T004 Play geometry tokens"

    $Notice = Get-Content -LiteralPath (Join-Path $RepoRoot "THIRD_PARTY_NOTICES.md") -Raw
    foreach ($Token in @(
        "egui, egui-winit, egui-wgpu",
        "winit",
        "wgpu",
        "pollster",
        "CPAL",
        "rusqlite / bundled SQLite"
    )) {
        if ($Notice -notmatch [regex]::Escape($Token)) {
            throw "THIRD_PARTY_NOTICES missing linked component record: $Token"
        }
    }
    Write-Host "[PASS] linked UI/GPU plus existing audio/storage third-party notice inventory"

    cargo test -p m32-ui
    if ($LASTEXITCODE -ne 0) {
        throw "m32-ui tests failed with exit code $LASTEXITCODE"
    }

    cargo test -p m32-display
    if ($LASTEXITCODE -ne 0) {
        throw "m32-display tests failed with exit code $LASTEXITCODE"
    }

    cargo check -p m32-desktop --target x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) {
        throw "m32-desktop Windows compile gate failed with exit code $LASTEXITCODE"
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
Write-Host "[PASS] T001 native Windows shell contract"
Write-Host "[PASS] T002 egui/wgpu renderer, resize and device-loss recovery boundary"
Write-Host "[PASS] T003 truthful 1000ms M32 boot ritual"
Write-Host "[PASS] T004 1040 + 16 + 264 First Playable composition skeleton"
Write-Host "[PASS] previous 0.0.6 Storage version-close regression chain"
Write-Host "[PASS] exact UI/GPU dependency pins"
Write-Host "[PASS] WIE and RustJava boundary unchanged"
Write-Host "[PASS] workspace fmt/clippy/test/check quality gates"
Write-Host "[PASS] git diff whitespace gate"
Write-Host ""
Write-Host "M32 0.1.0 First Playable Bundle A verification passed."
exit 0
