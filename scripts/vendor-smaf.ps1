$ErrorActionPreference = "Stop"

$repository = "https://github.com/dlunch/smaf.git"
$revision = "8009d78512fd121609a841f31aa527bf2a4af456"

$repoRoot = Split-Path -Parent $PSScriptRoot
$destination = Join-Path $repoRoot "third_party\smaf"
$tempRoot = Join-Path $env:TEMP ("m32-smaf-vendor-" + [Guid]::NewGuid().ToString("N"))

if (Test-Path -LiteralPath $destination) {
    throw "Destination already exists: $destination`nRefusing to overwrite vendored source."
}

try {
    New-Item -ItemType Directory -Path $tempRoot -Force | Out-Null
    $clonePath = Join-Path $tempRoot "source"

    Write-Host "Cloning SMAF upstream..."
    git -c core.autocrlf=false clone --quiet --no-checkout $repository $clonePath
    if ($LASTEXITCODE -ne 0) {
        throw "git clone failed with exit code $LASTEXITCODE"
    }

    git -C $clonePath -c core.autocrlf=false checkout --quiet --detach $revision
    if ($LASTEXITCODE -ne 0) {
        throw "git checkout failed with exit code $LASTEXITCODE"
    }

    $actualRevision = (git -C $clonePath rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0) {
        throw "git rev-parse failed with exit code $LASTEXITCODE"
    }

    if ($actualRevision -ne $revision) {
        throw "Unexpected SMAF revision. Expected $revision, got $actualRevision"
    }

    $requiredFiles = @(
        "LICENSE",
        "smaf\Cargo.toml",
        "smaf\src\lib.rs",
        "smaf_player\Cargo.toml",
        "smaf_player\src\lib.rs"
    )

    foreach ($relativePath in $requiredFiles) {
        $sourcePath = Join-Path $clonePath $relativePath
        if (-not (Test-Path -LiteralPath $sourcePath -PathType Leaf)) {
            throw "Required SMAF file is missing at pinned revision: $relativePath"
        }
    }

    New-Item -ItemType Directory -Path $destination -Force | Out-Null

    Copy-Item -LiteralPath (Join-Path $clonePath "LICENSE") `
        -Destination (Join-Path $destination "LICENSE")

    Copy-Item -LiteralPath (Join-Path $clonePath "smaf") `
        -Destination (Join-Path $destination "smaf") -Recurse

    Copy-Item -LiteralPath (Join-Path $clonePath "smaf_player") `
        -Destination (Join-Path $destination "smaf_player") -Recurse

    # Only production package sources are vendored. Remove upstream test directories if present.
    foreach ($testDirectory in @(
        (Join-Path $destination "smaf\tests"),
        (Join-Path $destination "smaf_player\tests")
    )) {
        if (Test-Path -LiteralPath $testDirectory) {
            Remove-Item -LiteralPath $testDirectory -Recurse -Force
        }
    }

    $provenance = [ordered]@{
        schema_version = 1
        repository = $repository
        revision = $revision
        components = @("smaf", "smaf_player")
        license = "MIT"
        copyright = "Copyright 2020 Inseok Lee"
        vendoring_reason = "Reproduce the SMAF revision recorded by the pinned WIE Cargo.lock."
    }

    $provenance |
        ConvertTo-Json -Depth 4 |
        Set-Content -LiteralPath (Join-Path $destination "M32_UPSTREAM.json") -Encoding utf8

    Write-Host ""
    Write-Host "[PASS] SMAF repository: $repository"
    Write-Host "[PASS] SMAF revision:   $revision"
    Write-Host "[PASS] Vendored path:   $destination"
    Write-Host "[PASS] Components:      smaf, smaf_player"
    Write-Host "SMAF vendoring completed."
}
finally {
    if (Test-Path -LiteralPath $tempRoot) {
        Remove-Item -LiteralPath $tempRoot -Recurse -Force
    }
}
