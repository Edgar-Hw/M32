$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$Jad = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\j2me-first-frame-running.jad"
$Jar = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\j2me-first-frame-running.jar"
$Source = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\src\m32\RunningMidlet.java"

$ExpectedJadSha = "e02fba5850f154913d1ed45d845c63f4cad53a2f1d66d348f5046a83c56a1ae7"
$ExpectedJarSha = "2521a16329f92ec6eaf07a2d9bc379713261c00146a722c14b5dd0aab5bb465d"

Write-Host "M32 First Frame Positive Boot Verification"
Write-Host "=========================================="

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-rustjava-system-loader-compat.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "RustJava system-loader compatibility verification failed with exit code $LASTEXITCODE"
}

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-wie-first-frame-bootable-fixture.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "T002 bootable fixture verification failed with exit code $LASTEXITCODE"
}

foreach ($Path in @($Jad, $Jar, $Source)) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Required T003 fixture file missing: $Path"
    }
}

$ActualJadSha = (Get-FileHash -LiteralPath $Jad -Algorithm SHA256).Hash.ToLowerInvariant()
$ActualJarSha = (Get-FileHash -LiteralPath $Jar -Algorithm SHA256).Hash.ToLowerInvariant()

if ($ActualJadSha -ne $ExpectedJadSha) {
    throw "T003 JAD SHA-256 '$ActualJadSha' does not match '$ExpectedJadSha'."
}
if ($ActualJarSha -ne $ExpectedJarSha) {
    throw "T003 JAR SHA-256 '$ActualJarSha' does not match '$ExpectedJarSha'."
}

$SourceText = [System.IO.File]::ReadAllText($Source)
if (-not $SourceText.Contains('System.out.println("M32_FIRST_FRAME_BOOT_OK")')) {
    throw "startApp stdout sentinel source is missing."
}

cargo test -p m32-wie-adapter tests::first_frame_running_fixture_contains_start_app_sentinel -- --exact
if ($LASTEXITCODE -ne 0) {
    throw "T003 fixture identity test failed with exit code $LASTEXITCODE"
}

cargo test -p m32-wie-adapter tests::first_frame_running_fixture_executes_start_app_and_reaches_running -- --exact
if ($LASTEXITCODE -ne 0) {
    throw "T003 positive boot smoke failed with exit code $LASTEXITCODE"
}

Write-Host ""
Write-Host "[PASS] deterministic positive-boot JAD/JAR"
Write-Host "[PASS] locked T003 fixture hashes"
Write-Host "[PASS] RustJar caller -> system URLClassLoader compatibility"
Write-Host "[PASS] guest JAR metadata queried through URLClassLoader"
Write-Host "[PASS] real RustJava guest class load"
Write-Host "[PASS] RunningMidlet constructor path"
Write-Host "[PASS] MIDlet.startApp execution sentinel"
Write-Host "[PASS] guest stdout -> M32 GuestOutputHost bridge"
Write-Host "[PASS] SessionState::Running after observed startApp"
Write-Host "[PASS] previous First Frame/Core Adapter verification chain"
Write-Host "First Frame positive boot verification passed."
exit 0
