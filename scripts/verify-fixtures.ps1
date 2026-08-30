$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$manifestPath = Join-Path $repoRoot "assets\fixtures\fixture-manifest.json"

if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
    throw "Fixture manifest not found: $manifestPath"
}

$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json

if ($manifest.schema_version -ne 1) {
    throw "Unsupported fixture manifest schema_version: $($manifest.schema_version)"
}

if (-not $manifest.fixtures -or $manifest.fixtures.Count -lt 1) {
    throw "Fixture manifest contains no fixtures."
}

$requiredFields = @(
    "id",
    "kind",
    "path",
    "license",
    "source",
    "sha256",
    "purpose",
    "redistribution_status"
)

$seenIds = @{}

foreach ($fixture in $manifest.fixtures) {
    foreach ($field in $requiredFields) {
        $property = $fixture.PSObject.Properties[$field]
        if ($null -eq $property -or [string]::IsNullOrWhiteSpace([string]$property.Value)) {
            throw "Fixture is missing required field '$field': $($fixture | ConvertTo-Json -Compress)"
        }
    }

    $id = [string]$fixture.id
    if ($id -notmatch '^[a-z0-9]+(?:-[a-z0-9]+)*$') {
        throw "Fixture id must be kebab-case ASCII: $id"
    }

    if ($seenIds.ContainsKey($id)) {
        throw "Duplicate fixture id: $id"
    }
    $seenIds[$id] = $true

    if ([string]$fixture.redistribution_status -ne "redistributable") {
        throw "Tracked fixture is not marked redistributable: $id"
    }

    $relativePath = ([string]$fixture.path).Replace("/", "\")
    if ([System.IO.Path]::IsPathRooted($relativePath)) {
        throw "Fixture path must be repository-relative: $id -> $relativePath"
    }

    $parts = $relativePath -split '[\\/]'
    if ($parts -contains "..") {
        throw "Fixture path traversal is forbidden: $id -> $relativePath"
    }

    if (-not $relativePath.StartsWith("assets\fixtures\")) {
        throw "Fixture must live below assets\\fixtures: $id -> $relativePath"
    }

    $fullPath = Join-Path $repoRoot $relativePath
    if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) {
        throw "Fixture file does not exist: $id -> $fullPath"
    }

    $actualHash = (Get-FileHash -LiteralPath $fullPath -Algorithm SHA256).Hash.ToLowerInvariant()
    $expectedHash = ([string]$fixture.sha256).ToLowerInvariant()

    if ($expectedHash -notmatch '^[0-9a-f]{64}$') {
        throw "Fixture sha256 must be 64 lowercase hex characters: $id"
    }

    if ($actualHash -ne $expectedHash) {
        throw "Fixture hash mismatch: $id`nExpected: $expectedHash`nActual:   $actualHash"
    }

    Write-Host "[PASS] $id  $actualHash"
}

Write-Host ""
Write-Host "Fixture manifest verification passed."
Write-Host "Schema: $($manifest.schema_version)"
Write-Host "Fixtures: $($manifest.fixtures.Count)"
exit 0
