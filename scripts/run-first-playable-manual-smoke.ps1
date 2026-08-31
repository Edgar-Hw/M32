$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$Evidence = Join-Path $RepoRoot "docs\spec\task-evidence\M32_0.1.0-T010_evidence.md"
$Jad = Join-Path $RepoRoot "apps\m32-desktop\test-fixtures\j2me-first-playable.jad"
$Jar = Join-Path $RepoRoot "apps\m32-desktop\test-fixtures\j2me-first-playable.jar"
$Stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogRoot = Join-Path $env:TEMP "M32-first-playable-manual-$Stamp"
New-Item -ItemType Directory -Force -Path $LogRoot | Out-Null

function Confirm-Pass([string]$Prompt) {
    $Answer = Read-Host "$Prompt [y/N]"
    return $Answer -match '^(?i:y|yes)$'
}

Write-Host "M32 0.1.0 T010 Windows Manual First Playable Smoke"
Write-Host "=================================================="
Write-Host "Logs: $LogRoot"
Write-Host ""
Write-Host "First, M32 will identify/test the physical Windows output device."

Push-Location $RepoRoot
try {
    $PreviousErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    & cargo run -p m32-audio --example windows_audio_smoke 2>&1 |
        Tee-Object -FilePath (Join-Path $LogRoot "audio-device.log")
    $AudioExitCode = $LASTEXITCODE
    $ErrorActionPreference = $PreviousErrorActionPreference
    if ($AudioExitCode -ne 0) {
        throw "Windows audio device smoke failed with exit code $AudioExitCode"
    }

    Write-Host ""
    Write-Host "RUN 1"
    Write-Host "- Wait for the First Playable game frame."
    Write-Host "- Press Right or Enter at least once. The green marker/bar must move."
    Write-Host "- A short guest-triggered tone should be audible for each action."
    Write-Host "- Remember the final marker position, then close the M32 window."

    $PreviousErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    & cargo run -p m32-desktop -- --jad $Jad --jar $Jar 2>&1 |
        Tee-Object -FilePath (Join-Path $LogRoot "first-launch.log")
    $FirstLaunchExitCode = $LASTEXITCODE
    $ErrorActionPreference = $PreviousErrorActionPreference
    if ($FirstLaunchExitCode -ne 0) {
        throw "First playable desktop launch failed with exit code $FirstLaunchExitCode"
    }

    Write-Host ""
    Write-Host "RUN 2"
    Write-Host "- M32 will relaunch against the same real application storage root."
    Write-Host "- Confirm the initial marker/bar position matches the saved RUN 1 position."
    Write-Host "- Press Right or Enter again and confirm visible input + guest tone."
    Write-Host "- Close the M32 window normally."

    $PreviousErrorActionPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    & cargo run -p m32-desktop -- --jad $Jad --jar $Jar 2>&1 |
        Tee-Object -FilePath (Join-Path $LogRoot "second-launch.log")
    $SecondLaunchExitCode = $LASTEXITCODE
    $ErrorActionPreference = $PreviousErrorActionPreference
    if ($SecondLaunchExitCode -ne 0) {
        throw "Second playable desktop launch failed with exit code $SecondLaunchExitCode"
    }
}
finally {
    Pop-Location
}

$FramePass = Confirm-Pass "Game frame was visible and Right/Enter changed the guest scene"
$AudioPass = Confirm-Pass "The guest action produced an audible short tone"
$RestorePass = Confirm-Pass "Second launch visibly restored the saved RUN 1 state"
$ExitPass = Confirm-Pass "Both windows closed cleanly without a panic/crash dialog"

if (-not ($FramePass -and $AudioPass -and $RestorePass -and $ExitPass)) {
    Write-Host ""
    Write-Host "M32 0.1.0 T010 manual smoke NOT PASSED. Evidence remains pending."
    exit 1
}

$Os = try {
    (Get-CimInstance Win32_OperatingSystem | Select-Object -First 1 -ExpandProperty Caption)
}
catch {
    [System.Environment]::OSVersion.VersionString
}
$AudioLog = Get-Content -LiteralPath (Join-Path $LogRoot "audio-device.log") -Raw
$AudioSummary = (($AudioLog -split "`r?`n") | Where-Object { $_ -match '48000|48000Hz|device|Device|Speakers|Headphones' } | Select-Object -Last 6) -join "`n"
if ([string]::IsNullOrWhiteSpace($AudioSummary)) {
    $AudioSummary = "Physical CPAL smoke exited 0; see $LogRoot\audio-device.log"
}
$Now = Get-Date -Format "yyyy-MM-dd HH:mm:ss K"

$Body = @"
# M32 0.1.0-T010 Evidence — Windows Manual First Playable Smoke

Status: PASS

MANUAL_WINDOWS_FIRST_PLAYABLE_SMOKE: PASS

Observed at: $Now
Observed OS: $Os

Exact commands:
  cargo run -p m32-audio --example windows_audio_smoke
  cargo run -p m32-desktop -- --jad apps\m32-desktop\test-fixtures\j2me-first-playable.jad --jar apps\m32-desktop\test-fixtures\j2me-first-playable.jar
  close, then run the same desktop command again

Human observations:
  PASS game frame visible
  PASS Right/Enter changed real guest-visible state
  PASS guest-triggered short tone audible
  PASS second application/runtime launch visibly restored saved state
  PASS normal window close / runtime release

Physical audio/device observation:
$AudioSummary

Raw manual logs outside repository:
$LogRoot
"@

Set-Content -LiteralPath $Evidence -Value $Body -Encoding UTF8
Write-Host ""
Write-Host "M32 0.1.0 T010 manual smoke PASSED."
Write-Host "Evidence updated: $Evidence"
exit 0
