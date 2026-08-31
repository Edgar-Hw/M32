$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$AdapterCargo = Join-Path $RepoRoot "crates\m32-wie-adapter\Cargo.toml"
$AdapterSource = Join-Path $RepoRoot "crates\m32-wie-adapter\src\lib.rs"
$RmsJad = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\j2me-rms-persistence.jad"
$RmsJar = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\j2me-rms-persistence.jar"
$RmsSource = Join-Path $RepoRoot "crates\m32-wie-adapter\test-fixtures\src\m32\RmsPersistenceMidlet.java"

Write-Host "M32 0.0.6 Storage Bundle B Functional Verification"
Write-Host "==================================================="

foreach ($Path in @($AdapterCargo, $AdapterSource, $RmsJad, $RmsJar, $RmsSource)) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Storage Bundle B file missing: $Path"
    }
}

$ExpectedJadSha = "fa9610eec08acc1e62d0340d6c6a3d46547b9b343c83e9c204d99fc1bf129597"
$ExpectedJarSha = "a0fe3a4bffb117bee1ae9eb57924f01263738f9e7e2143daf146bcea71fdbad6"
$ActualJadSha = (Get-FileHash -LiteralPath $RmsJad -Algorithm SHA256).Hash.ToLowerInvariant()
$ActualJarSha = (Get-FileHash -LiteralPath $RmsJar -Algorithm SHA256).Hash.ToLowerInvariant()

if ($ActualJadSha -ne $ExpectedJadSha) {
    throw "RMS JAD SHA-256 mismatch: $ActualJadSha"
}
if ($ActualJarSha -ne $ExpectedJarSha) {
    throw "RMS JAR SHA-256 mismatch: $ActualJarSha"
}

$CargoText = [System.IO.File]::ReadAllText($AdapterCargo)
$SourceText = [System.IO.File]::ReadAllText($AdapterSource)
$JavaText = [System.IO.File]::ReadAllText($RmsSource)

if (-not $CargoText.Contains('[dev-dependencies]')) {
    throw "m32-wie-adapter dev-dependency section missing"
}
if (-not $CargoText.Contains('m32-storage = { path = "../m32-storage" }')) {
    throw "m32-wie-adapter m32-storage dev dependency missing"
}

$RustMarkers = @(
    'RMS_PERSISTENCE_APP_ID: &str = "M32 RMS Persistence"',
    'RMS_PERSISTENCE_STORE_NAME: &str = "m32-rms"',
    'persistent_storage_platform_hosts',
    'persistent_storage_hosts_are_accepted_by_wie_platform',
    'rms_persistence_fixture_locks_real_record_store_contract',
    'real_j2me_rms_survives_session_and_storage_rebuild',
    'persistent_wie_filesystem_survives_platform_rebuild'
)
foreach ($Marker in $RustMarkers) {
    if (-not $SourceText.Contains($Marker)) {
        throw "Storage Bundle B Rust marker missing: $Marker"
    }
}

$JavaMarkers = @(
    'RecordStore.openRecordStore("m32-rms", true)',
    'store.getNumRecords() == 0',
    'store.addRecord(expected, 0, expected.length)',
    'store.getRecord(1)',
    'M32_RMS_SAVED;',
    'M32_RMS_LOADED_OK;'
)
foreach ($Marker in $JavaMarkers) {
    if (-not $JavaText.Contains($Marker)) {
        throw "Storage Bundle B Java marker missing: $Marker"
    }
}

Push-Location $RepoRoot
try {
    $MetadataJson = cargo metadata --format-version 1 --no-deps
    if ($LASTEXITCODE -ne 0) {
        throw "cargo metadata failed with exit code $LASTEXITCODE"
    }

    $Metadata = $MetadataJson | ConvertFrom-Json
    $AdapterPackage = $Metadata.packages | Where-Object { $_.name -eq "m32-wie-adapter" }
    if ($null -eq $AdapterPackage) {
        throw "m32-wie-adapter missing from workspace metadata"
    }

    $StorageDependencies = @(
        $AdapterPackage.dependencies |
            Where-Object { $_.name -eq "m32-storage" }
    )

    if ($StorageDependencies.Count -ne 1) {
        throw "m32-wie-adapter must have exactly one m32-storage dependency; found $($StorageDependencies.Count)"
    }

    $StorageDependency = $StorageDependencies[0]
    if ($StorageDependency.kind -ne "dev") {
        throw "m32-storage must remain a dev-only adapter dependency; found kind '$($StorageDependency.kind)'"
    }
    if ($null -ne $StorageDependency.source) {
        throw "m32-wie-adapter -> m32-storage dev dependency must remain workspace-local/path-based"
    }

    cargo test -p m32-wie-adapter tests::persistent_storage_hosts_are_accepted_by_wie_platform -- --exact
    if ($LASTEXITCODE -ne 0) {
        throw "T006 persistent WIE platform assembly test failed"
    }

    cargo test -p m32-wie-adapter tests::rms_persistence_fixture_locks_real_record_store_contract -- --exact
    if ($LASTEXITCODE -ne 0) {
        throw "T007 deterministic J2ME RMS fixture contract failed"
    }

    cargo test -p m32-wie-adapter tests::real_j2me_rms_survives_session_and_storage_rebuild -- --exact
    if ($LASTEXITCODE -ne 0) {
        throw "T008 real J2ME RMS restart persistence test failed"
    }

    cargo test -p m32-wie-adapter tests::persistent_wie_filesystem_survives_platform_rebuild -- --exact
    if ($LASTEXITCODE -ne 0) {
        throw "T009 persistent WIE filesystem restart test failed"
    }
}
finally {
    Pop-Location
}

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-storage-bundle-a.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "Previous Storage Bundle A / Audio version-close regression chain failed with exit code $LASTEXITCODE"
}

Write-Host ""
Write-Host "[PASS] T006 concrete persistent RMS/filesystem hosts assemble through WIE Platform"
Write-Host "[PASS] T006 m32-storage remains dev-only in m32-wie-adapter"
Write-Host "[PASS] T007 deterministic Java 8 RecordStore fixture"
Write-Host "[PASS] T007 locked RMS JAD/JAR SHA-256 identities"
Write-Host "[PASS] T008 real Java RecordStore save reaches persistent M32 SQLite host"
Write-Host "[PASS] T008 emulator + storage reconstruction reloads exact saved guest bytes"
Write-Host "[PASS] T008 stable MIDlet-Name application namespace and exact 8-byte usage"
Write-Host "[PASS] T009 WIE filesystem adapter writes persistent M32 disk host"
Write-Host "[PASS] T009 rebuilt WIE platform reads exact persistent guest file bytes"
Write-Host "[PASS] previous Storage Bundle A / Audio / Input / First Frame chain"
Write-Host ""
Write-Host "M32 0.0.6 Storage Bundle B functional verification passed."
exit 0
