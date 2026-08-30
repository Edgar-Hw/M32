$ErrorActionPreference = "Stop"

$expectedWieRepository = "https://github.com/dlunch/wie.git"
$expectedWieRevision = "f0513eb758c02736981f545ad030eed937d55f3e"

$expectedSmafRepository = "https://github.com/dlunch/smaf.git"
$expectedSmafRevision = "8009d78512fd121609a841f31aa527bf2a4af456"
$expectedTracingVersion = "0.1.41"
$expectedTracingAttributesVersion = "0.1.28"

$repoRoot = Split-Path -Parent $PSScriptRoot
$smafRoot = Join-Path $repoRoot "third_party\smaf"
$smafProvenancePath = Join-Path $smafRoot "M32_UPSTREAM.json"

if (-not (Test-Path -LiteralPath $smafProvenancePath -PathType Leaf)) {
    throw "Vendored SMAF provenance is missing. Run scripts\vendor-smaf.ps1 first."
}

$smafProvenance = Get-Content -LiteralPath $smafProvenancePath -Raw | ConvertFrom-Json

if ([string]$smafProvenance.repository -ne $expectedSmafRepository) {
    throw "Unexpected vendored SMAF repository: $($smafProvenance.repository)"
}

if ([string]$smafProvenance.revision -ne $expectedSmafRevision) {
    throw "Unexpected vendored SMAF revision: $($smafProvenance.revision)"
}

$metadataJson = cargo metadata --format-version 1
if ($LASTEXITCODE -ne 0) {
    throw "cargo metadata failed with exit code $LASTEXITCODE"
}

$metadata = $metadataJson | ConvertFrom-Json

$workspaceWie = $metadata.metadata.m32.upstream.wie
if ($null -eq $workspaceWie) {
    throw "workspace.metadata.m32.upstream.wie is missing."
}

if ([string]$workspaceWie.repository -ne $expectedWieRepository) {
    throw "Unexpected workspace WIE repository: $($workspaceWie.repository)"
}

if ([string]$workspaceWie.revision -ne $expectedWieRevision) {
    throw "Unexpected workspace WIE revision: $($workspaceWie.revision)"
}

$workspaceSmaf = $metadata.metadata.m32.upstream.smaf
if ($null -eq $workspaceSmaf) {
    throw "workspace.metadata.m32.upstream.smaf is missing."
}

if ([string]$workspaceSmaf.repository -ne $expectedSmafRepository) {
    throw "Unexpected workspace SMAF repository: $($workspaceSmaf.repository)"
}

if ([string]$workspaceSmaf.revision -ne $expectedSmafRevision) {
    throw "Unexpected workspace SMAF revision: $($workspaceSmaf.revision)"
}


$tracingPackages = @($metadata.packages | Where-Object { $_.name -eq "tracing" })
if ($tracingPackages.Count -ne 1) {
    throw "Expected exactly one resolved tracing package, found $($tracingPackages.Count)."
}

if ([string]$tracingPackages[0].version -ne $expectedTracingVersion) {
    throw "Unexpected tracing version: $($tracingPackages[0].version)"
}

$tracingAttributePackages = @($metadata.packages | Where-Object { $_.name -eq "tracing-attributes" })
if ($tracingAttributePackages.Count -ne 1) {
    throw "Expected exactly one resolved tracing-attributes package, found $($tracingAttributePackages.Count)."
}

if ([string]$tracingAttributePackages[0].version -ne $expectedTracingAttributesVersion) {
    throw "Unexpected tracing-attributes version: $($tracingAttributePackages[0].version)"
}

$wiePackages = @($metadata.packages | Where-Object { $_.name -eq "wie_backend" })
if ($wiePackages.Count -ne 1) {
    throw "Expected exactly one wie_backend package, found $($wiePackages.Count)."
}

$wieBackend = $wiePackages[0]
$wieSource = [string]$wieBackend.source

if ([string]::IsNullOrWhiteSpace($wieSource)) {
    throw "wie_backend is not resolved from a Git source."
}

if ($wieSource -notmatch [regex]::Escape("git+https://github.com/dlunch/wie.git")) {
    throw "wie_backend resolved from an unexpected repository: $wieSource"
}

if ($wieSource -notmatch [regex]::Escape("rev=$expectedWieRevision")) {
    throw "wie_backend source does not contain the pinned rev query: $wieSource"
}

if (-not $wieSource.EndsWith("#$expectedWieRevision")) {
    throw "wie_backend resolved commit does not equal the pinned revision: $wieSource"
}

$adapterPackages = @($metadata.packages | Where-Object { $_.name -eq "m32-wie-adapter" })
if ($adapterPackages.Count -ne 1) {
    throw "Expected exactly one m32-wie-adapter package, found $($adapterPackages.Count)."
}

$adapterDependency = @(
    $adapterPackages[0].dependencies | Where-Object { $_.name -eq "wie_backend" }
)

if ($adapterDependency.Count -ne 1) {
    throw "m32-wie-adapter must have exactly one direct wie_backend dependency."
}

foreach ($packageName in @("smaf", "smaf_player")) {
    $packages = @($metadata.packages | Where-Object { $_.name -eq $packageName })
    if ($packages.Count -ne 1) {
        throw "Expected exactly one $packageName package, found $($packages.Count)."
    }

    $package = $packages[0]

    if ($null -ne $package.source) {
        throw "$packageName must resolve from the vendored path patch, got source: $($package.source)"
    }

    $manifestPath = [System.IO.Path]::GetFullPath([string]$package.manifest_path)
    $expectedPrefix = [System.IO.Path]::GetFullPath($smafRoot) + [System.IO.Path]::DirectorySeparatorChar

    if (-not $manifestPath.StartsWith($expectedPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "$packageName manifest is outside the vendored SMAF root: $manifestPath"
    }

    Write-Host "[PASS] $packageName path patch: $manifestPath"
}

Write-Host ""
Write-Host "[PASS] WIE repository:  $expectedWieRepository"
Write-Host "[PASS] WIE revision:    $expectedWieRevision"
Write-Host "[PASS] WIE Cargo source: $wieSource"
Write-Host "[PASS] SMAF repository: $expectedSmafRepository"
Write-Host "[PASS] SMAF revision:   $expectedSmafRevision"
Write-Host "[PASS] tracing:         $expectedTracingVersion"
Write-Host "[PASS] tracing-attributes: $expectedTracingAttributesVersion"
Write-Host "[PASS] m32-wie-adapter directly depends on wie_backend"
Write-Host "WIE upstream verification passed."
exit 0
