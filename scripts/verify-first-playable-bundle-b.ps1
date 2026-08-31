$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$EvidenceRoot = Join-Path $RepoRoot "docs\spec\task-evidence"
$FixtureRoot = Join-Path $RepoRoot "apps\m32-desktop\test-fixtures"

Write-Host "M32 0.1.0 First Playable Bundle B Verification"
Write-Host "================================================"

for ($Task = 5; $Task -le 8; $Task++) {
    $TaskId = "T{0:D3}" -f $Task
    $Evidence = Join-Path $EvidenceRoot "M32_0.1.0-$TaskId`_evidence.md"
    if (-not (Test-Path -LiteralPath $Evidence -PathType Leaf)) {
        throw "Missing First Playable Bundle B evidence: $Evidence"
    }
    Write-Host "[PASS] evidence 0.1.0-$TaskId ($(Split-Path $Evidence -Leaf))"
}

# Bundle A is the previous canonical chain and already includes the workspace
# fmt/clippy/test/check + git diff + RustJava quality gates on the current tree.
powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-first-playable-bundle-a.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "Previous First Playable Bundle A canonical chain failed with exit code $LASTEXITCODE"
}

Push-Location $RepoRoot
try {
    $Composition = Get-Content -LiteralPath "apps\m32-desktop\src\composition.rs" -Raw
    $Audio = Get-Content -LiteralPath "crates\m32-audio\src\lib.rs" -Raw
    $Main = Get-Content -LiteralPath "apps\m32-desktop\src\main.rs" -Raw
    $FixtureSource = Get-Content -LiteralPath "apps\m32-desktop\test-fixtures\src\m32\FirstPlayableMidlet.java" -Raw

    foreach ($Token in @(
        "CpalOutputStream",
        "CpalOutputStream::open_default",
        "_audio_stream",
        "RealtimeGuestAudioHost",
        "RealtimeAudioBridge"
    )) {
        if ($Composition -notmatch [regex]::Escape($Token)) {
            throw "T005 realtime audio composition token missing: $Token"
        }
    }
    foreach ($Token in @(
        "OUTPUT_SAMPLE_RATE_HZ: u32 = 48_000",
        "TARGET_LATENCY_FRAMES",
        "PAUSE_FADE_FRAMES",
        "runtime.try_lock()",
        "data.fill(0.0)"
    )) {
        if ($Audio -notmatch [regex]::Escape($Token)) {
            throw "T005 canonical audio token missing: $Token"
        }
    }
    Write-Host "[PASS] T005 live CPAL stream ownership + canonical realtime audio engine"

    foreach ($Token in @(
        "PersistentGuestStorage::open(m32_root)",
        "storage.database_repository()",
        "storage.filesystem()"
    )) {
        if ($Composition -notmatch [regex]::Escape($Token)) {
            throw "T006 persistent composition token missing: $Token"
        }
    }
    if ($Main -notmatch [regex]::Escape("desktop::run(paths.root.clone(), launch)")) {
        throw "T006 desktop must supply real M32 application root to composition"
    }
    Write-Host "[PASS] T006 real M32 root -> existing PersistentGuestStorage wiring"

    $Jad = Join-Path $FixtureRoot "j2me-first-playable.jad"
    $Jar = Join-Path $FixtureRoot "j2me-first-playable.jar"
    if (-not (Test-Path $Jad) -or -not (Test-Path $Jar)) {
        throw "T007 First Playable JAD/JAR fixture missing"
    }
    $JadHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $Jad).Hash.ToLowerInvariant()
    $JarHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $Jar).Hash.ToLowerInvariant()
    if ($JadHash -ne "a30f6dacf6b5eb8ebc0cbabc0e9008d60a0c38918ac6960b88b57f182a76e0d2") {
        throw "T007 JAD hash drift: $JadHash"
    }
    if ($JarHash -ne "b8eb1efefb3e54783492d72f3255d33b7ad66d7527d1305e08e1e0e49731b6b5") {
        throw "T007 JAR hash drift: $JarHash"
    }
    foreach ($Token in @(
        "RecordStore.openRecordStore",
        "Manager.createPlayer",
        "0x4D, 0x4D, 0x4D, 0x44",
        "(byte) 0x90",
        "repaint()",
        "M32_FP_RUNNING:",
        "M32_FP_INPUT:",
        "M32_FP_AUDIO:",
        "M32_FP_SAVED:"
    )) {
        if ($FixtureSource -notmatch [regex]::Escape($Token)) {
            throw "T007 First Playable Java fixture token missing: $Token"
        }
    }
    Write-Host "[PASS] T007 deterministic copyright-clean First Playable JAD/JAR + locked hashes"

    cargo test -p m32-desktop composition::tests::composed_storage_uses_real_m32_root_and_survives_runtime_rebuild
    if ($LASTEXITCODE -ne 0) {
        throw "T006 composed persistent storage test failed with exit code $LASTEXITCODE"
    }

    cargo test -p m32-desktop composition::tests::first_playable_fixture_integrates_frame_input_audio_and_rms
    if ($LASTEXITCODE -ne 0) {
        throw "T008 integrated First Playable loop test failed with exit code $LASTEXITCODE"
    }

    cargo check -p m32-desktop --target x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) {
        throw "T005 Windows desktop/audio compile gate failed with exit code $LASTEXITCODE"
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
Write-Host "[PASS] T005 realtime Windows audio output wiring"
Write-Host "[PASS] T006 persistent RMS/filesystem product composition"
Write-Host "[PASS] T007 deterministic M32 First Playable Java mini-game fixture"
Write-Host "[PASS] T008 integrated frame/input/audio/storage loop"
Write-Host "[PASS] previous Bundle A canonical + workspace quality chain"
Write-Host ""
Write-Host "M32 0.1.0 First Playable Bundle B verification passed."
exit 0
