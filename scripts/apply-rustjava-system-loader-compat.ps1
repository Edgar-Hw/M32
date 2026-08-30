$ErrorActionPreference = "Stop"

$ExpectedRevision = "ba5797b8eb4cf376fdd63129903d319d1d7acf98"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$VendorRoot = Join-Path $RepoRoot "third_party\rustjava"
$RevisionFile = Join-Path $VendorRoot "M32_VENDOR_REVISION.txt"
$Target = Join-Path $VendorRoot "jvm\src\jvm.rs"

Write-Host "M32 RustJava System Loader Compatibility Patch"
Write-Host "=============================================="

if (-not (Test-Path -LiteralPath $RevisionFile -PathType Leaf)) {
    throw "RustJava vendor revision marker is missing."
}

$ActualRevision = (Get-Content -LiteralPath $RevisionFile -Raw).Trim()
if ($ActualRevision -ne $ExpectedRevision) {
    throw "RustJava base revision '$ActualRevision' does not match '$ExpectedRevision'."
}

if (-not (Test-Path -LiteralPath $Target -PathType Leaf)) {
    throw "Vendored RustJava JVM source is missing: $Target"
}

$Raw = [System.IO.File]::ReadAllText($Target)
$Normalized = $Raw.Replace("`r`n", "`n")

$FunctionMarker = "    async fn current_class_loader(&self) -> Result<Box<dyn ClassInstance>> {"
$PatchMarker = "M32 compatibility: RustJar callers resolve application classes through the system loader"

if (-not $Normalized.Contains($FunctionMarker)) {
    throw "Pinned current_class_loader function marker is missing. Refusing to modify unexpected RustJava source."
}

if ($Normalized.Contains($PatchMarker)) {
    Write-Host "[PASS] compatibility patch already applied"
    Write-Host "[PASS] exact RustJava base revision $ExpectedRevision"
    exit 0
}

$OldFragment = @'
            if let Some(x) = calling_class_class_loader {
                Ok(x)
            } else {
'@.Replace("`r`n", "`n")

$NewFragment = @'
            if let Some(x) = calling_class_class_loader {
                // M32 compatibility: RustJar callers resolve application classes through the system loader.
                //
                // The system loader is URLClassLoader(parent = RustJarClassLoader). A native RustJar
                // class such as net/wie/Launcher must therefore use the system loader when resolving
                // an application JAR class. Otherwise the current RustJarClassLoader skips every
                // non-.rustjar classpath entry and guest classes are never queried from the JAR.
                if x.class_definition().name() == "org/rustjava/lang/RustJarClassLoader" {
                    let system_class_loader = JavaLangClassLoader::get_system_class_loader(self).await?;
                    return Ok(system_class_loader);
                }

                Ok(x)
            } else {
'@.Replace("`r`n", "`n")

$Count = 0
$SearchFrom = 0
while ($true) {
    $Index = $Normalized.IndexOf($OldFragment, $SearchFrom, [System.StringComparison]::Ordinal)
    if ($Index -lt 0) {
        break
    }
    $Count++
    $SearchFrom = $Index + $OldFragment.Length
}

if ($Count -ne 1) {
    throw "Expected exactly one pinned loader branch fragment, found $Count. Refusing to patch unexpected RustJava source."
}

$Patched = $Normalized.Replace($OldFragment, $NewFragment)

if (-not $Patched.Contains($PatchMarker)) {
    throw "Compatibility patch marker was not produced."
}
if (-not $Patched.Contains('x.class_definition().name() == "org/rustjava/lang/RustJarClassLoader"')) {
    throw "RustJarClassLoader compatibility condition was not produced."
}

$Utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllText($Target, $Patched, $Utf8NoBom)

Write-Host "[PASS] exact RustJava base revision $ExpectedRevision"
Write-Host "[PASS] current_class_loader function marker"
Write-Host "[PASS] pinned loader branch matched exactly once"
Write-Host "[PASS] RustJar caller -> system URLClassLoader compatibility path applied"
Write-Host "RustJava system loader compatibility patch applied."
exit 0
