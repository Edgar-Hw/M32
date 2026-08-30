$ErrorActionPreference = "Stop"

$ExpectedRevision = "ba5797b8eb4cf376fdd63129903d319d1d7acf98"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$VendorRoot = Join-Path $RepoRoot "third_party\rustjava"

Write-Host "M32 RustJava Vendor Verification"
Write-Host "================================"

if (-not (Test-Path -LiteralPath $VendorRoot -PathType Container)) {
    throw "RustJava vendor directory missing. Run scripts\vendor-rustjava.ps1 first."
}

$RevisionFile = Join-Path $VendorRoot "M32_VENDOR_REVISION.txt"
if (-not (Test-Path -LiteralPath $RevisionFile -PathType Leaf)) {
    throw "RustJava vendor revision marker missing."
}

$ActualRevision = (Get-Content -LiteralPath $RevisionFile -Raw).Trim()
if ($ActualRevision -ne $ExpectedRevision) {
    throw "RustJava vendor revision '$ActualRevision' does not match '$ExpectedRevision'."
}

$ExpectedPackages = @(
    "classfile",
    "java_class_proto",
    "java_constants",
    "java_runtime",
    "jvm",
    "jvm_rust"
)

foreach ($Package in $ExpectedPackages) {
    $Manifest = Join-Path $VendorRoot "$Package\Cargo.toml"
    if (-not (Test-Path -LiteralPath $Manifest -PathType Leaf)) {
        throw "Missing vendored package manifest: $Manifest"
    }
}

if (Test-Path -LiteralPath (Join-Path $VendorRoot ".git")) {
    throw "Vendored RustJava must not contain a nested .git directory."
}

if (-not (Test-Path -LiteralPath (Join-Path $VendorRoot "LICENSE") -PathType Leaf)) {
    throw "Vendored RustJava LICENSE missing."
}


$WorkspaceManifest = Join-Path $VendorRoot "Cargo.toml"
if (-not (Test-Path -LiteralPath $WorkspaceManifest -PathType Leaf)) {
    throw "Vendored RustJava workspace root Cargo.toml missing."
}

$WorkspaceText = Get-Content -LiteralPath $WorkspaceManifest -Raw
if ($WorkspaceText -notmatch '\[workspace\.package\]') {
    throw "Vendored RustJava workspace.package table missing."
}
if ($WorkspaceText -notmatch 'license\s*=\s*"MIT"') {
    throw "Vendored RustJava workspace package license is not the expected MIT value."
}

cargo metadata `
    --manifest-path $WorkspaceManifest `
    --no-deps `
    --format-version 1 | Out-Null
if ($LASTEXITCODE -ne 0) {
    throw "Vendored RustJava nested workspace metadata failed with exit code $LASTEXITCODE"
}

Write-Host "[PASS] vendor directory"
Write-Host "[PASS] exact RustJava revision"
Write-Host "[PASS] required package manifests"
Write-Host "[PASS] java_class_proto restored from locked revision"
Write-Host "[PASS] upstream LICENSE"
Write-Host "[PASS] no nested .git"
Write-Host "[PASS] RustJava nested workspace manifest"
Write-Host "[PASS] RustJava workspace.package inheritance metadata"
Write-Host "[PASS] RustJava nested workspace Cargo metadata"
Write-Host "RustJava vendor verification passed."
exit 0
