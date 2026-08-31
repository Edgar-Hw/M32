$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$InputCargo = Join-Path $RepoRoot "crates\m32-input\Cargo.toml"
$InputSource = Join-Path $RepoRoot "crates\m32-input\src\lib.rs"

Write-Host "M32 0.0.4 Input Bundle C Functional Verification"
Write-Host "================================================"

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-wie-input-bundle-b.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "Input Bundle B verification failed with exit code $LASTEXITCODE"
}

if (-not (Test-Path -LiteralPath $InputCargo -PathType Leaf)) {
    throw "m32-input Cargo.toml missing"
}
if (-not (Test-Path -LiteralPath $InputSource -PathType Leaf)) {
    throw "m32-input source missing"
}

$CargoText = [System.IO.File]::ReadAllText($InputCargo)
$SourceText = [System.IO.File]::ReadAllText($InputSource)

if (-not $CargoText.Contains('m32-emulator-api = { path = "../m32-emulator-api" }')) {
    throw "m32-input -> m32-emulator-api dependency boundary missing"
}

$RequiredMarkers = @(
    "KEY_REPEAT_DELAY_MS: u64 = 350",
    "KEY_REPEAT_HZ: u64 = 12",
    "MAX_HELD_GUEST_KEYS: usize = 6",
    "HeldKeyLimitReached",
    "AlreadyHeld",
    "repeats_due"
)

foreach ($Marker in $RequiredMarkers) {
    if (-not $SourceText.Contains($Marker)) {
        throw "Required input policy marker missing: $Marker"
    }
}

cargo test -p m32-input
if ($LASTEXITCODE -ne 0) {
    throw "m32-input policy suite failed with exit code $LASTEXITCODE"
}

cargo test -p m32-emulator-api
if ($LASTEXITCODE -ne 0) {
    throw "m32-emulator-api regression suite failed with exit code $LASTEXITCODE"
}

cargo test -p m32-wie-adapter
if ($LASTEXITCODE -ne 0) {
    throw "m32-wie-adapter regression suite failed with exit code $LASTEXITCODE"
}

Write-Host ""
Write-Host "[PASS] T008 repeat delay exactly 350ms"
Write-Host "[PASS] T008 repeat frequency exactly 12Hz"
Write-Host "[PASS] T008 repeat schedule derived from press origin without cumulative drift"
Write-Host "[PASS] T008 delayed poll catches up deterministically"
Write-Host "[PASS] T008 release stops future repeats"
Write-Host "[PASS] T009 maximum held guest keys exactly 6"
Write-Host "[PASS] T009 seventh distinct key is rejected without guest event"
Write-Host "[PASS] T009 duplicate key-down does not duplicate held state"
Write-Host "[PASS] T009 released capacity can accept a new key"
Write-Host "[PASS] deterministic held-key repeat ordering"
Write-Host "[PASS] m32-input -> m32-emulator-api workspace dependency boundary"
Write-Host "[PASS] previous Bundle B / Bundle A / First Frame chain"
Write-Host "M32 0.0.4 Input Bundle C functional verification passed."
exit 0
