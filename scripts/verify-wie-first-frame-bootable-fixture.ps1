$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$Jad = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\j2me-first-frame-boot.jad"
$Jar = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\j2me-first-frame-boot.jar"
$Source = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\src\m32\FirstFrameMidlet.java"
$Stub = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\src\javax\microedition\midlet\MIDlet.java"

$ExpectedJadSha = "a30e92605f738bdca0eeb2f7c694b87aa21fbf212215c652287fb50c8e9f745d"
$ExpectedJarSha = "9ff5c3a86d913f7f49453312773af6b1cc43f595674e1206bcb64268dc573b3c"

Write-Host "M32 Deterministic Bootable MIDlet Fixture Verification"
Write-Host "======================================================"

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-wie-first-frame-jad-jar-launch.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "T001 JAD+JAR launch verification failed with exit code $LASTEXITCODE"
}

foreach ($Path in @($Jad, $Jar, $Source, $Stub)) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Required First Frame fixture file missing: $Path"
    }
}

$ActualJadSha = (Get-FileHash -LiteralPath $Jad -Algorithm SHA256).Hash.ToLowerInvariant()
$ActualJarSha = (Get-FileHash -LiteralPath $Jar -Algorithm SHA256).Hash.ToLowerInvariant()

if ($ActualJadSha -ne $ExpectedJadSha) {
    throw "JAD SHA-256 '$ActualJadSha' does not match '$ExpectedJadSha'."
}
if ($ActualJarSha -ne $ExpectedJarSha) {
    throw "JAR SHA-256 '$ActualJarSha' does not match '$ExpectedJarSha'."
}

$JadText = [System.IO.File]::ReadAllText($Jad)
if (-not $JadText.Contains("MIDlet-1: M32 First Frame,,m32.FirstFrameMidlet")) {
    throw "JAD MIDlet-1 contract missing."
}

$JarBytes = [System.IO.File]::ReadAllBytes($Jar)
$JarAscii = [System.Text.Encoding]::ASCII.GetString($JarBytes)
if (-not $JarAscii.Contains("m32/FirstFrameMidlet.class")) {
    throw "JAR class entry name missing."
}
if (-not $JarAscii.Contains("m32/FirstFrameMidlet")) {
    throw "JAR class internal name missing."
}

cargo test -p m32-wie-adapter tests::first_frame_boot_fixture_has_expected_container_and_class_identity -- --exact
if ($LASTEXITCODE -ne 0) {
    throw "Fixture identity test failed with exit code $LASTEXITCODE"
}

cargo test -p m32-wie-adapter tests::first_frame_boot_fixture_constructs_ready_j2me_session -- --exact
if ($LASTEXITCODE -ne 0) {
    throw "Fixture constructor test failed with exit code $LASTEXITCODE"
}

Write-Host ""
Write-Host "[PASS] deterministic JAD fixture"
Write-Host "[PASS] deterministic JAR fixture"
Write-Host "[PASS] locked JAD/JAR SHA-256"
Write-Host "[PASS] M32-owned MIDlet source provenance"
Write-Host "[PASS] explicit m32.FirstFrameMidlet entry point"
Write-Host "[PASS] Java 8 classfile identity"
Write-Host "[PASS] Ready pinned J2ME constructor path"
Write-Host "[PASS] previous First Frame/Core Adapter verification chain"
Write-Host "Deterministic bootable MIDlet fixture verification passed."
exit 0
