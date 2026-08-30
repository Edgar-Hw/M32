$ErrorActionPreference = "Stop"

$ExpectedRevision = "ba5797b8eb4cf376fdd63129903d319d1d7acf98"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$VendorRoot = Join-Path $RepoRoot "third_party\rustjava"
$RevisionFile = Join-Path $VendorRoot "M32_VENDOR_REVISION.txt"
$Target = Join-Path $VendorRoot "jvm\src\jvm.rs"

Write-Host "M32 RustJava System Loader Patch Verification"
Write-Host "============================================="

if (-not (Test-Path -LiteralPath $RevisionFile -PathType Leaf)) {
    throw "RustJava vendor revision marker is missing."
}
if ((Get-Content -LiteralPath $RevisionFile -Raw).Trim() -ne $ExpectedRevision) {
    throw "Unexpected RustJava base revision."
}
if (-not (Test-Path -LiteralPath $Target -PathType Leaf)) {
    throw "Vendored jvm.rs missing."
}

$Text = [System.IO.File]::ReadAllText($Target)

$Required = @(
    "M32 compatibility: RustJar callers resolve application classes through the system loader",
    'x.class_definition().name() == "org/rustjava/lang/RustJarClassLoader"',
    "JavaLangClassLoader::get_system_class_loader(self).await?"
)

foreach ($Needle in $Required) {
    if (-not $Text.Contains($Needle)) {
        throw "Required RustJava compatibility source fragment missing: $Needle"
    }
}

Write-Host "[PASS] exact RustJava base revision"
Write-Host "[PASS] RustJarClassLoader caller detection"
Write-Host "[PASS] system URLClassLoader fallback"
Write-Host "[PASS] vendored source compatibility patch"
Write-Host "RustJava system loader patch verification passed."
exit 0
