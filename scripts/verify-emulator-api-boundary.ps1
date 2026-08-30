$ErrorActionPreference = "Stop"

$metadataJson = cargo metadata --format-version 1
if ($LASTEXITCODE -ne 0) {
    throw "cargo metadata failed with exit code $LASTEXITCODE"
}

$metadata = $metadataJson | ConvertFrom-Json

function Get-WorkspacePackage([string]$name) {
    $packages = @($metadata.packages | Where-Object { $_.name -eq $name })
    if ($packages.Count -ne 1) {
        throw "Expected exactly one package named '$name', found $($packages.Count)."
    }
    return $packages[0]
}

$api = Get-WorkspacePackage "m32-emulator-api"
$adapter = Get-WorkspacePackage "m32-wie-adapter"
$desktop = Get-WorkspacePackage "m32-desktop"

$apiWieDependencies = @(
    $api.dependencies | Where-Object {
        $_.name -eq "wie_backend" -or
        ([string]$_.source -match "github\.com/dlunch/wie")
    }
)

if ($apiWieDependencies.Count -ne 0) {
    throw "m32-emulator-api must not depend on WIE."
}

$adapterApiDependencies = @(
    $adapter.dependencies | Where-Object { $_.name -eq "m32-emulator-api" }
)
if ($adapterApiDependencies.Count -ne 1) {
    throw "m32-wie-adapter must directly depend on m32-emulator-api."
}

$adapterWieDependencies = @(
    $adapter.dependencies | Where-Object { $_.name -eq "wie_backend" }
)
if ($adapterWieDependencies.Count -ne 1) {
    throw "m32-wie-adapter must directly depend on wie_backend."
}

$desktopWieDependencies = @(
    $desktop.dependencies | Where-Object {
        $_.name -eq "wie_backend" -or
        ([string]$_.source -match "github\.com/dlunch/wie")
    }
)
if ($desktopWieDependencies.Count -ne 0) {
    throw "m32-desktop must not directly depend on WIE."
}

Write-Host "[PASS] m32-emulator-api has no WIE dependency"
Write-Host "[PASS] m32-wie-adapter -> m32-emulator-api"
Write-Host "[PASS] m32-wie-adapter -> wie_backend"
Write-Host "[PASS] m32-desktop has no direct WIE dependency"
Write-Host "Emulator API boundary verification passed."
exit 0
