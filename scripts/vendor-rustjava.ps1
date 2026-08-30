$ErrorActionPreference = "Stop"

$Repository = "https://github.com/dlunch/RustJava.git"
$Revision = "ba5797b8eb4cf376fdd63129903d319d1d7acf98"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$ThirdPartyRoot = Join-Path $RepoRoot "third_party"
$Target = Join-Path $ThirdPartyRoot "rustjava"
$RevisionFile = Join-Path $Target "M32_VENDOR_REVISION.txt"
$ProvenanceFile = Join-Path $Target "M32_VENDOR_PROVENANCE.md"

$ExpectedPackages = @(
    "classfile",
    "java_class_proto",
    "java_constants",
    "java_runtime",
    "jvm",
    "jvm_rust"
)

function Assert-VendorLayout {
    param([string]$Root)

    foreach ($Package in $ExpectedPackages) {
        $Manifest = Join-Path $Root "$Package\Cargo.toml"
        if (-not (Test-Path -LiteralPath $Manifest -PathType Leaf)) {
            throw "Missing expected RustJava package manifest: $Manifest"
        }
    }

    $License = Join-Path $Root "LICENSE"
    if (-not (Test-Path -LiteralPath $License -PathType Leaf)) {
        throw "Missing RustJava LICENSE at $License"
    }

    $NestedGit = Join-Path $Root ".git"
    if (Test-Path -LiteralPath $NestedGit) {
        throw "Nested .git directory must not remain in vendored RustJava source."
    }
}

if (Test-Path -LiteralPath $Target) {
    if (-not (Test-Path -LiteralPath $RevisionFile -PathType Leaf)) {
        throw "third_party\rustjava already exists without M32 vendor revision metadata. Refusing to overwrite it."
    }

    $ExistingRevision = (Get-Content -LiteralPath $RevisionFile -Raw).Trim()
    if ($ExistingRevision -ne $Revision) {
        throw "Existing RustJava vendor revision is '$ExistingRevision'; expected '$Revision'."
    }

    Assert-VendorLayout -Root $Target
    Write-Host "[PASS] RustJava vendor already exists at locked revision $Revision"
    exit 0
}

New-Item -ItemType Directory -Path $ThirdPartyRoot -Force | Out-Null

$Temp = Join-Path $ThirdPartyRoot (".rustjava-fetch-" + [Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $Temp -Force | Out-Null

try {
    git -C $Temp init --quiet
    if ($LASTEXITCODE -ne 0) {
        throw "git init failed with exit code $LASTEXITCODE"
    }

    git -C $Temp remote add origin $Repository
    if ($LASTEXITCODE -ne 0) {
        throw "git remote add failed with exit code $LASTEXITCODE"
    }

    git -C $Temp fetch --depth 1 origin $Revision
    if ($LASTEXITCODE -ne 0) {
        throw "git fetch of locked RustJava revision failed with exit code $LASTEXITCODE"
    }

    git -C $Temp checkout --detach --quiet FETCH_HEAD
    if ($LASTEXITCODE -ne 0) {
        throw "git checkout of locked RustJava revision failed with exit code $LASTEXITCODE"
    }

    $ResolvedRevision = (git -C $Temp rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0) {
        throw "git rev-parse failed with exit code $LASTEXITCODE"
    }
    if ($ResolvedRevision -ne $Revision) {
        throw "Fetched RustJava revision '$ResolvedRevision' does not match locked revision '$Revision'."
    }

    $GitDir = Join-Path $Temp ".git"
    Remove-Item -LiteralPath $GitDir -Recurse -Force

    Set-Content -LiteralPath (Join-Path $Temp "M32_VENDOR_REVISION.txt") -Value $Revision -NoNewline

    $Provenance = @"
# M32 Vendored RustJava Provenance

Repository: $Repository
Revision: $Revision
Mode: vendored-path-patch
Reason: reproduce the RustJava revision recorded by pinned WIE Cargo.lock
Task: 0.0.2-T011

This directory is a source snapshot, not a Git submodule or nested repository.
The upstream LICENSE is preserved in this directory.
"@
    Set-Content -LiteralPath (Join-Path $Temp "M32_VENDOR_PROVENANCE.md") -Value $Provenance -NoNewline

    Assert-VendorLayout -Root $Temp

    Move-Item -LiteralPath $Temp -Destination $Target
}
finally {
    if (Test-Path -LiteralPath $Temp) {
        Remove-Item -LiteralPath $Temp -Recurse -Force
    }
}

Assert-VendorLayout -Root $Target

Write-Host ""
Write-Host "[PASS] RustJava repository fetched"
Write-Host "[PASS] exact revision $Revision"
Write-Host "[PASS] java_class_proto package present"
Write-Host "[PASS] required RustJava package manifests present"
Write-Host "[PASS] upstream LICENSE preserved"
Write-Host "[PASS] nested .git removed"
Write-Host "RustJava vendoring completed."
exit 0
