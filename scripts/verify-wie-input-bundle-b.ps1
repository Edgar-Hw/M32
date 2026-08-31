$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$Jad = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\j2me-input-key-observer.jad"
$Jar = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\j2me-input-key-observer.jar"

Write-Host "M32 0.0.4 Input Bundle B Verification"
Write-Host "====================================="

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-wie-input-bundle-a.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "Input Bundle A verification failed with exit code $LASTEXITCODE"
}

if (-not (Test-Path -LiteralPath $Jad -PathType Leaf)) {
    throw "Input key observer JAD missing: $Jad"
}
if (-not (Test-Path -LiteralPath $Jar -PathType Leaf)) {
    throw "Input key observer JAR missing: $Jar"
}

$JadSha = (Get-FileHash -LiteralPath $Jad -Algorithm SHA256).Hash.ToLowerInvariant()
$JarSha = (Get-FileHash -LiteralPath $Jar -Algorithm SHA256).Hash.ToLowerInvariant()

if ($JadSha -ne "4edee7aaf35396e1965e1e5c6a2e4e0e9e22f3c94dd033638ca9be9e2aaf9825") {
    throw "Input key observer JAD SHA-256 mismatch: $JadSha"
}
if ($JarSha -ne "be7cb8fa6933ac2b1ebd1303e3e9e549a8e731a6b0b0d9a2e44f630b22df7ca2") {
    throw "Input key observer JAR SHA-256 mismatch: $JarSha"
}

cargo test -p m32-wie-adapter tests::input_key_observer_fixture_locks_guest_callback_contract -- --exact
if ($LASTEXITCODE -ne 0) { throw "T004 fixture contract test failed" }

cargo test -p m32-wie-adapter tests::input_key_observer_fixture_constructs_ready_j2me_session -- --exact
if ($LASTEXITCODE -ne 0) { throw "T004 Ready constructor test failed" }

cargo test -p m32-wie-adapter tests::input_key_down_reaches_real_canvas_key_pressed -- --exact
if ($LASTEXITCODE -ne 0) { throw "T005 real keyPressed callback test failed" }

cargo test -p m32-wie-adapter tests::input_key_up_and_repeat_reach_real_canvas_callbacks -- --exact
if ($LASTEXITCODE -ne 0) { throw "T006 real release/repeat callback test failed" }

cargo test -p m32-wie-adapter tests::input_all_24_keys_reach_real_canvas_with_exact_midp_codes -- --exact
if ($LASTEXITCODE -ne 0) { throw "T007 full MIDP key matrix test failed" }

Write-Host ""
Write-Host "[PASS] T004 deterministic KeyMidlet/KeyCanvas fixture"
Write-Host "[PASS] T004 locked JAD/JAR SHA-256"
Write-Host "[PASS] T004 real pinned J2ME Ready constructor"
Write-Host "[PASS] T005 KeyDown -> real Canvas.keyPressed"
Write-Host "[PASS] T005 Up -> MIDP 141"
Write-Host "[PASS] T006 KeyUp -> real Canvas.keyReleased"
Write-Host "[PASS] T006 KeyRepeat -> real Canvas.keyRepeated"
Write-Host "[PASS] T007 all 24 M32 keys -> exact MIDP codes"
Write-Host "[PASS] real WIE EventQueue -> Display -> Canvas virtual dispatch"
Write-Host "[PASS] previous Bundle A / First Frame chain"
Write-Host "M32 0.0.4 Input Bundle B verification passed."
exit 0
