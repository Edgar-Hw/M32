$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$Jad = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\j2me-audio-mmapi.jad"
$Jar = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\j2me-audio-mmapi.jar"

Write-Host "M32 0.0.5 Audio Bundle B Functional Verification"
Write-Host "================================================"

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-wie-audio-bundle-a.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "Audio Bundle A verification failed with exit code $LASTEXITCODE"
}

if (-not (Test-Path -LiteralPath $Jad -PathType Leaf)) {
    throw "MMAPI JAD missing: $Jad"
}
if (-not (Test-Path -LiteralPath $Jar -PathType Leaf)) {
    throw "MMAPI JAR missing: $Jar"
}

$JadSha = (Get-FileHash -LiteralPath $Jad -Algorithm SHA256).Hash.ToLowerInvariant()
$JarSha = (Get-FileHash -LiteralPath $Jar -Algorithm SHA256).Hash.ToLowerInvariant()

if ($JadSha -ne "1d8f5de025d5c201df5992e28070037343049b544ee6d93571030046c9f827d1") {
    throw "MMAPI JAD SHA-256 mismatch: $JadSha"
}
if ($JarSha -ne "ed88d129e90388aa045bebef5a4389e7751fa88890ad8f607b35761ba536dbc7") {
    throw "MMAPI JAR SHA-256 mismatch: $JarSha"
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

    $ApiDependency = $AudioPackage.dependencies |
        Where-Object { $_.name -eq "m32-emulator-api" }
    if ($null -eq $ApiDependency -or $null -ne $ApiDependency.source) {
        throw "m32-audio -> m32-emulator-api workspace dependency boundary missing"
    }

    $CpalDependency = $AudioPackage.dependencies |
        Where-Object { $_.name -eq "cpal" }
    if ($null -eq $CpalDependency) {
        throw "Windows CPAL dependency missing"
    }
    if ($CpalDependency.req -ne "=0.18.2") {
        throw "CPAL must be pinned exactly to =0.18.2; found $($CpalDependency.req)"
    }

    cargo test -p m32-wie-adapter tests::audio_mmapi_fixture_locks_guest_manager_player_contract -- --exact
    if ($LASTEXITCODE -ne 0) { throw "T006 MMAPI fixture contract failed" }

    cargo test -p m32-wie-adapter tests::real_j2me_mmapi_start_stop_reaches_m32_audio_host -- --exact
    if ($LASTEXITCODE -ne 0) { throw "T007 real J2ME MMAPI integration failed" }

    cargo test -p m32-audio bundle_b_tests
    if ($LASTEXITCODE -ne 0) { throw "T008/T009 deterministic audio runtime tests failed" }

    cargo check -p m32-audio --example windows_audio_smoke
    if ($LASTEXITCODE -ne 0) { throw "T009 Windows audio smoke example compile failed" }
}
finally {
    Pop-Location
}

Write-Host ""
Write-Host "[PASS] T006 deterministic AudioMidlet MMAPI/SMAF fixture"
Write-Host "[PASS] T006 locked JAD/JAR hashes and Java 8 fixture identity"
Write-Host "[PASS] T007 real guest Manager.createPlayer -> SmafPlayer path"
Write-Host "[PASS] T007 real guest Player.start/stop -> M32 GuestAudioHost"
Write-Host "[PASS] T008 exact timed Wave scheduling at 48kHz"
Write-Host "[PASS] T008 audio-handle Stop lifecycle"
Write-Host "[PASS] T008 deterministic sequence repeat"
Write-Host "[PASS] T008 baseline MIDI NoteOn/NoteOff rendering"
Write-Host "[PASS] T009 exact 80ms / 3840-frame pause fade"
Write-Host "[PASS] T009 Windows CPAL output example compiles"
Write-Host "[PASS] CPAL pinned exactly to 0.18.2 on Windows"
Write-Host "[PASS] previous Audio Bundle A / Input / First Frame chain"
Write-Host ""
Write-Host "AUTOMATED GATE PASSED."
Write-Host "T009 still requires the local audible device smoke:"
Write-Host "cargo run -p m32-audio --example windows_audio_smoke"
Write-Host ""
Write-Host "M32 0.0.5 Audio Bundle B functional verification passed."
exit 0
