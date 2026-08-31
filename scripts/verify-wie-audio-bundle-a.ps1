$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$AudioCargo = Join-Path $RepoRoot "crates\m32-audio\Cargo.toml"
$AudioSource = Join-Path $RepoRoot "crates\m32-audio\src\lib.rs"

Write-Host "M32 0.0.5 Audio Bundle A Verification"
Write-Host "====================================="

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-wie-input-version-close.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "Previous 0.0.4 Input version-close chain failed with exit code $LASTEXITCODE"
}

if (-not (Test-Path -LiteralPath $AudioCargo -PathType Leaf)) {
    throw "m32-audio Cargo.toml missing"
}
if (-not (Test-Path -LiteralPath $AudioSource -PathType Leaf)) {
    throw "m32-audio source missing"
}

$CargoText = [System.IO.File]::ReadAllText($AudioCargo)
$SourceText = [System.IO.File]::ReadAllText($AudioSource)

if (-not $CargoText.Contains('m32-emulator-api = { path = "../m32-emulator-api" }')) {
    throw "m32-audio -> m32-emulator-api workspace dependency missing"
}

$Markers = @(
    "OUTPUT_SAMPLE_RATE_HZ: u32 = 48_000",
    "OUTPUT_CHANNELS: u8 = 2",
    "TARGET_LATENCY_MS: u32 = 60",
    "PAUSE_FADE_MS: u32 = 80",
    "decode_i16_interleaved_to_stereo",
    "resample_stereo_to_output_rate",
    "mix_stereo_clips",
    "BufferedGuestAudioHost"
)

foreach ($Marker in $Markers) {
    if (-not $SourceText.Contains($Marker)) {
        throw "Required Audio Bundle A marker missing: $Marker"
    }
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

    $NormalDependencies = @(
        $AudioPackage.dependencies |
            Where-Object { $null -eq $_.kind -or $_.kind -eq "normal" }
    )

    if ($NormalDependencies.Count -ne 1) {
        throw "m32-audio must have exactly one normal dependency in Bundle A; found $($NormalDependencies.Count)"
    }

    if ($NormalDependencies[0].name -ne "m32-emulator-api") {
        throw "m32-audio Bundle A dependency must be m32-emulator-api"
    }

    if ($null -ne $NormalDependencies[0].source) {
        throw "m32-audio -> m32-emulator-api must remain workspace-local/path-based"
    }

    cargo test -p m32-audio
    if ($LASTEXITCODE -ne 0) {
        throw "m32-audio Bundle A tests failed with exit code $LASTEXITCODE"
    }
}
finally {
    Pop-Location
}

Write-Host ""
Write-Host "[PASS] T001 48kHz f32 stereo canonical output contract"
Write-Host "[PASS] T001 exact 60ms/2880-frame latency target"
Write-Host "[PASS] T001 exact 80ms/3840-frame pause-fade contract"
Write-Host "[PASS] T002 mono/stereo i16 normalization contract"
Write-Host "[PASS] T002 malformed/unsupported PCM rejection boundary"
Write-Host "[PASS] T003 deterministic linear resampling to 48kHz"
Write-Host "[PASS] T003 zero-rate failure boundary"
Write-Host "[PASS] T004 deterministic stereo summing/saturation/silence"
Write-Host "[PASS] T005 thread-safe GuestAudioHost FIFO ingress"
Write-Host "[PASS] T005 Play/Stop/MIDI/Wave payload preservation"
Write-Host "[PASS] m32-audio has exactly one workspace-local production dependency"
Write-Host "[PASS] previous 0.0.4 Input version-close chain"
Write-Host "M32 0.0.5 Audio Bundle A verification passed."
exit 0
