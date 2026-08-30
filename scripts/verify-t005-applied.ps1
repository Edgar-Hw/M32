$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot

$requiredFiles = @(
    "scripts\verify-wie-display-bridge.ps1",
    "docs\test\WIE_DISPLAY_BRIDGE.md",
    "crates\m32-emulator-api\src\lib.rs",
    "crates\m32-wie-adapter\Cargo.toml",
    "crates\m32-wie-adapter\src\lib.rs"
)

foreach ($relativePath in $requiredFiles) {
    $path = Join-Path $repoRoot $relativePath
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "T005 required file is missing: $relativePath"
    }
}

$apiPath = Join-Path $repoRoot "crates\m32-emulator-api\src\lib.rs"
$adapterCargoPath = Join-Path $repoRoot "crates\m32-wie-adapter\Cargo.toml"
$adapterLibPath = Join-Path $repoRoot "crates\m32-wie-adapter\src\lib.rs"
$rootCargoPath = Join-Path $repoRoot "Cargo.toml"

$api = Get-Content -LiteralPath $apiPath -Raw
$adapterCargo = Get-Content -LiteralPath $adapterCargoPath -Raw
$adapterLib = Get-Content -LiteralPath $adapterLibPath -Raw
$rootCargo = Get-Content -LiteralPath $rootCargoPath -Raw

$checks = [ordered]@{
    "DisplayHost contract" = $api.Contains("pub trait DisplayHost")
    "RgbaFrame contract" = $api.Contains("pub struct RgbaFrame")
    "DisplaySize contract" = $api.Contains("pub struct DisplaySize")
    "WieScreenAdapter" = $adapterLib.Contains("pub struct WieScreenAdapter")
    "adapter tracing dependency" = $adapterCargo.Contains("tracing.workspace = true")
    "adapter wie_util dependency" = $adapterCargo.Contains("wie_util.workspace = true")
    "workspace wie_util pin" = $rootCargo.Contains("wie_util = { git = ""https://github.com/dlunch/wie.git""")
    "display verifier" = Test-Path -LiteralPath (Join-Path $repoRoot "scripts\verify-wie-display-bridge.ps1")
}

foreach ($entry in $checks.GetEnumerator()) {
    if (-not $entry.Value) {
        throw "T005 apply check failed: $($entry.Key)"
    }

    Write-Host "[PASS] $($entry.Key)"
}

$apiTestCount = ([regex]::Matches($api, "#\[test\]")).Count
$adapterTestCount = ([regex]::Matches($adapterLib, "#\[test\]")).Count

if ($apiTestCount -ne 12) {
    throw "Expected 12 m32-emulator-api tests in T005 source, found $apiTestCount."
}

if ($adapterTestCount -ne 11) {
    throw "Expected 11 m32-wie-adapter tests in T005 source, found $adapterTestCount."
}

Write-Host "[PASS] m32-emulator-api source test count: $apiTestCount"
Write-Host "[PASS] m32-wie-adapter source test count: $adapterTestCount"
Write-Host "T005 source application verification passed."
exit 0
