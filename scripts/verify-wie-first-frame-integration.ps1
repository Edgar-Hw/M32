$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$AdapterSource = Join-Path $RepoRoot "crates\m32-wie-adapter\src\lib.rs"
$RunningJad = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\j2me-first-frame-running.jad"
$RunningJar = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\j2me-first-frame-running.jar"
$PaintJad = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\j2me-first-frame-paint.jad"
$PaintJar = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\j2me-first-frame-paint.jar"

$ExpectedRunningJadSha = "e02fba5850f154913d1ed45d845c63f4cad53a2f1d66d348f5046a83c56a1ae7"
$ExpectedRunningJarSha = "2521a16329f92ec6eaf07a2d9bc379713261c00146a722c14b5dd0aab5bb465d"
$ExpectedPaintJadSha = "7fe94a7cb014c40367ee2139d25cf4e1c3cde37d8e9d73bcb1277a4a1f9efc33"
$ExpectedPaintJarSha = "717063ece69306b9338da722d57f14815987c6a044f738da3b2e95373a9f8b5a"

Write-Host "M32 First-frame Canonical Integration Verification"
Write-Host "=================================================="

foreach ($Path in @($AdapterSource, $RunningJad, $RunningJar, $PaintJad, $PaintJar)) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Required First Frame integration file missing: $Path"
    }
}

$ActualRunningJadSha = (Get-FileHash -LiteralPath $RunningJad -Algorithm SHA256).Hash.ToLowerInvariant()
$ActualRunningJarSha = (Get-FileHash -LiteralPath $RunningJar -Algorithm SHA256).Hash.ToLowerInvariant()
$ActualPaintJadSha = (Get-FileHash -LiteralPath $PaintJad -Algorithm SHA256).Hash.ToLowerInvariant()
$ActualPaintJarSha = (Get-FileHash -LiteralPath $PaintJar -Algorithm SHA256).Hash.ToLowerInvariant()

if ($ActualRunningJadSha -ne $ExpectedRunningJadSha) {
    throw "Running JAD SHA-256 mismatch: $ActualRunningJadSha"
}
if ($ActualRunningJarSha -ne $ExpectedRunningJarSha) {
    throw "Running JAR SHA-256 mismatch: $ActualRunningJarSha"
}
if ($ActualPaintJadSha -ne $ExpectedPaintJadSha) {
    throw "Paint JAD SHA-256 mismatch: $ActualPaintJadSha"
}
if ($ActualPaintJarSha -ne $ExpectedPaintJarSha) {
    throw "Paint JAR SHA-256 mismatch: $ActualPaintJarSha"
}

$SourceText = [System.IO.File]::ReadAllText($AdapterSource)
$RequiredMarkers = @(
    "FirstFrameCaptureDisplayHost",
    "DeterministicAdvancingClock",
    "FirstFrameWaitError",
    "SessionFault",
    "Timeout",
    "first_frame_paint_fixture_ticks_until_guest_frame_is_captured",
    "first_frame_paint_fixture_locks_exact_rgba8_dimensions_and_content",
    "first_frame_wait_times_out_cleanly_when_running_midlet_never_paints",
    "first_frame_wait_reports_backend_fault_before_timeout"
)

foreach ($Marker in $RequiredMarkers) {
    if (-not $SourceText.Contains($Marker)) {
        throw "Required First Frame integration marker missing: $Marker"
    }
}

Write-Host "[PASS] locked RunningMidlet JAD/JAR hashes"
Write-Host "[PASS] locked PaintMidlet/PaintCanvas JAD/JAR hashes"
Write-Host "[PASS] canonical capture/clock/wait source markers"
Write-Host "[PASS] success/timeout/fault integration test markers"

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-wie-first-frame-timeout-failure.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "T008 timeout/failure verification chain failed with exit code $LASTEXITCODE"
}

cargo test -p m32-wie-adapter
if ($LASTEXITCODE -ne 0) {
    throw "Full m32-wie-adapter suite failed with exit code $LASTEXITCODE"
}

cargo test -p m32-emulator-api
if ($LASTEXITCODE -ne 0) {
    throw "Full m32-emulator-api suite failed with exit code $LASTEXITCODE"
}

Write-Host ""
Write-Host "[PASS] SUCCESS branch: real guest-generated exact RGBA8 frame"
Write-Host "[PASS] TIMEOUT branch: healthy Running guest without frame"
Write-Host "[PASS] FAULT branch: backend failure before timeout"
Write-Host "[PASS] exact 176x220 / 154880-byte / 38720-pixel content contract"
Write-Host "[PASS] capture host first-frame immutability"
Write-Host "[PASS] deterministic executor-time repaint scheduling"
Write-Host "[PASS] locked First Frame fixture identities"
Write-Host "[PASS] complete T001-T008 verification chain"
Write-Host "[PASS] full m32-wie-adapter regression suite"
Write-Host "[PASS] full m32-emulator-api regression suite"
Write-Host "First-frame canonical integration verification passed."
exit 0
