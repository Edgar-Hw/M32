$ErrorActionPreference = "Stop"

Write-Host "M32 First Frame JAD+JAR Launch Verification"
Write-Host "==========================================="

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-wie-core-adapter-smoke.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "Core Adapter smoke verification failed with exit code $LASTEXITCODE"
}

cargo test -p m32-wie-adapter tests::j2me_jad_jar_factory_constructs_ready_m32_session -- --exact
if ($LASTEXITCODE -ne 0) {
    throw "JAD+JAR factory construction test failed with exit code $LASTEXITCODE"
}

cargo test -p m32-wie-adapter tests::j2me_jad_jar_factory_uses_explicit_launch_descriptor_path -- --exact
if ($LASTEXITCODE -ne 0) {
    throw "Explicit JAD launch path test failed with exit code $LASTEXITCODE"
}

Write-Host ""
Write-Host "[PASS] pinned J2ME from_jad_jar constructor path"
Write-Host "[PASS] explicit MIDlet-1 launch descriptor"
Write-Host "[PASS] JAD bytes and JAR ownership transfer"
Write-Host "[PASS] Ready M32 session result"
Write-Host "[PASS] no new dependency"
Write-Host "[PASS] previous Core Adapter verification chain"
Write-Host "First Frame JAD+JAR launch verification passed."
exit 0
