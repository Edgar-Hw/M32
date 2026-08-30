$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$RootManifest = Join-Path $RepoRoot "Cargo.toml"
$VendorManifest = Join-Path $RepoRoot "third_party\rustjava\Cargo.toml"

Write-Host "M32 RustJava Workspace Boundary Verification"
Write-Host "============================================"

$RootText = Get-Content -LiteralPath $RootManifest -Raw
if ($RootText -notmatch 'exclude\s*=\s*\["third_party/rustjava"\]') {
    throw "M32 workspace must exclude third_party/rustjava."
}

if (-not (Test-Path -LiteralPath $VendorManifest -PathType Leaf)) {
    throw "RustJava nested workspace manifest missing."
}

cargo metadata `
    --manifest-path $VendorManifest `
    --no-deps `
    --format-version 1 | Out-Null
if ($LASTEXITCODE -ne 0) {
    throw "RustJava nested workspace metadata failed with exit code $LASTEXITCODE"
}

Write-Host "[PASS] M32 workspace excludes vendored RustJava"
Write-Host "[PASS] vendored RustJava retains its own workspace root"
Write-Host "[PASS] RustJava workspace inheritance parses independently"
Write-Host "RustJava workspace boundary verification passed."
exit 0
