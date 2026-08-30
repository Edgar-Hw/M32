$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$Jad = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\j2me-first-frame-paint.jad"
$Jar = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\j2me-first-frame-paint.jar"
$MidletSource = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\src\m32\PaintMidlet.java"
$CanvasSource = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\src\m32\PaintCanvas.java"

$ExpectedJadSha = "7fe94a7cb014c40367ee2139d25cf4e1c3cde37d8e9d73bcb1277a4a1f9efc33"
$ExpectedJarSha = "717063ece69306b9338da722d57f14815987c6a044f738da3b2e95373a9f8b5a"

Write-Host "M32 Deterministic Canvas/Paint Fixture Verification"
Write-Host "==================================================="

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-wie-first-frame-capture-host.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "T004 capture-host verification failed with exit code $LASTEXITCODE"
}

foreach ($Path in @($Jad, $Jar, $MidletSource, $CanvasSource)) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Required T005 fixture file missing: $Path"
    }
}

$ActualJadSha = (Get-FileHash -LiteralPath $Jad -Algorithm SHA256).Hash.ToLowerInvariant()
$ActualJarSha = (Get-FileHash -LiteralPath $Jar -Algorithm SHA256).Hash.ToLowerInvariant()

if ($ActualJadSha -ne $ExpectedJadSha) {
    throw "T005 JAD SHA-256 '$ActualJadSha' does not match '$ExpectedJadSha'."
}
if ($ActualJarSha -ne $ExpectedJarSha) {
    throw "T005 JAR SHA-256 '$ActualJarSha' does not match '$ExpectedJarSha'."
}

$MidletText = [System.IO.File]::ReadAllText($MidletSource)
$CanvasText = [System.IO.File]::ReadAllText($CanvasSource)

if (-not $MidletText.Contains("Display.getDisplay(this).setCurrent(new PaintCanvas());")) {
    throw "T005 Display.setCurrent launch contract missing."
}
if (-not $CanvasText.Contains("graphics.setColor(0x0E1114);")) {
    throw "T005 BG0 paint contract missing."
}
if (-not $CanvasText.Contains("graphics.fillRect(0, 0, 176, 220);")) {
    throw "T005 full-frame paint contract missing."
}
if (-not $CanvasText.Contains("graphics.setColor(0xD14A36);")) {
    throw "T005 marker color contract missing."
}
if (-not $CanvasText.Contains("graphics.fillRect(0, 0, 16, 16);")) {
    throw "T005 marker rectangle contract missing."
}

cargo test -p m32-wie-adapter tests::first_frame_paint_fixture_locks_canvas_and_pixel_pattern_contract -- --exact
if ($LASTEXITCODE -ne 0) {
    throw "T005 Canvas/paint identity test failed with exit code $LASTEXITCODE"
}

cargo test -p m32-wie-adapter tests::first_frame_paint_fixture_constructs_ready_j2me_session -- --exact
if ($LASTEXITCODE -ne 0) {
    throw "T005 constructor test failed with exit code $LASTEXITCODE"
}

Write-Host ""
Write-Host "[PASS] deterministic PaintMidlet/PaintCanvas JAD/JAR"
Write-Host "[PASS] locked T005 JAD/JAR SHA-256"
Write-Host "[PASS] explicit m32.PaintMidlet launch entry"
Write-Host "[PASS] Display.getDisplay -> setCurrent Canvas contract"
Write-Host "[PASS] 176x220 M32 BG0 full-frame paint contract"
Write-Host "[PASS] 16x16 M32 RED marker paint contract"
Write-Host "[PASS] Ready pinned J2ME constructor path"
Write-Host "[PASS] previous capture-host/positive-boot verification chain"
Write-Host "Deterministic Canvas/Paint fixture verification passed."
exit 0
