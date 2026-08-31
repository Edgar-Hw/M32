$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$CargoPath = Join-Path $RepoRoot "Cargo.toml"
$StorageCargo = Join-Path $RepoRoot "crates\m32-storage\Cargo.toml"
$StorageSource = Join-Path $RepoRoot "crates\m32-storage\src\lib.rs"

Write-Host "M32 0.0.6 Storage Bundle A Verification"
Write-Host "======================================="

powershell -ExecutionPolicy Bypass -File "$PSScriptRoot\verify-wie-audio-version-close.ps1"
if ($LASTEXITCODE -ne 0) {
    throw "Previous 0.0.5 Audio version-close chain failed with exit code $LASTEXITCODE"
}

if (-not (Test-Path -LiteralPath $StorageCargo -PathType Leaf)) {
    throw "m32-storage Cargo.toml missing"
}

if (-not (Test-Path -LiteralPath $StorageSource -PathType Leaf)) {
    throw "m32-storage source missing"
}

$RootCargoText = [System.IO.File]::ReadAllText($CargoPath)
if (-not $RootCargoText.Contains('"crates/m32-storage"')) {
    throw "m32-storage is not a root workspace member; run scripts/apply-storage-bundle-a.ps1"
}

$StorageCargoText = [System.IO.File]::ReadAllText($StorageCargo)
$StorageSourceText = [System.IO.File]::ReadAllText($StorageSource)

if (-not $StorageCargoText.Contains('m32-emulator-api = { path = "../m32-emulator-api" }')) {
    throw "m32-storage -> m32-emulator-api workspace dependency missing"
}

if (-not $StorageCargoText.Contains('rusqlite = { version = "=0.37.0", features = ["bundled"] }')) {
    throw "rusqlite exact =0.37.0 bundled dependency missing"
}

$Markers = @(
    'STORAGE_DATABASE_FILE_NAME: &str = "storage.sqlite3"',
    'GUEST_FILES_DIRECTORY_NAME: &str = "guest-files"',
    'STORAGE_SCHEMA_VERSION: i64 = 1',
    'SQLITE_BUSY_TIMEOUT_MS: u64 = 2_000',
    'PRAGMA journal_mode = WAL',
    'foreign_keys',
    'guest_databases',
    'guest_records',
    'SqliteGuestDatabaseRepository',
    'DiskGuestFilesystem',
    'PersistentGuestStorage',
    'encode_component',
    'matches!(component, "." | "..")'
)

foreach ($Marker in $Markers) {
    if (-not $StorageSourceText.Contains($Marker)) {
        throw "Required Storage Bundle A marker missing: $Marker"
    }
}

Push-Location $RepoRoot
try {
    $MetadataJson = cargo metadata --format-version 1 --no-deps
    if ($LASTEXITCODE -ne 0) {
        throw "cargo metadata failed with exit code $LASTEXITCODE"
    }

    $Metadata = $MetadataJson | ConvertFrom-Json
    $StoragePackage = $Metadata.packages |
        Where-Object { $_.name -eq "m32-storage" }

    if ($null -eq $StoragePackage) {
        throw "m32-storage missing from workspace metadata"
    }

    $ApiDependency = @(
        $StoragePackage.dependencies |
            Where-Object { $_.name -eq "m32-emulator-api" }
    )

    if ($ApiDependency.Count -ne 1) {
        throw "m32-storage must have exactly one m32-emulator-api dependency"
    }

    if ($null -ne $ApiDependency[0].source) {
        throw "m32-storage -> m32-emulator-api must remain workspace-local/path-based"
    }

    $RusqliteDependency = @(
        $StoragePackage.dependencies |
            Where-Object { $_.name -eq "rusqlite" }
    )

    if ($RusqliteDependency.Count -ne 1) {
        throw "m32-storage must have exactly one rusqlite dependency"
    }

    if ($RusqliteDependency[0].req -ne "=0.37.0") {
        throw "rusqlite must remain pinned exactly to =0.37.0; found $($RusqliteDependency[0].req)"
    }

    cargo test -p m32-storage
    if ($LASTEXITCODE -ne 0) {
        throw "m32-storage Bundle A tests failed with exit code $LASTEXITCODE"
    }
}
finally {
    Pop-Location
}

Write-Host ""
Write-Host "[PASS] T001 %LOCALAPPDATA%\M32-compatible storage root layout"
Write-Host "[PASS] T001 SQLite WAL / FK / 2000ms busy-timeout / schema-v1 policy"
Write-Host "[PASS] T001 rusqlite pinned exactly to 0.37.0 with bundled SQLite"
Write-Host "[PASS] T002 persistent RMS repository and record lifecycle"
Write-Host "[PASS] T002 monotonic non-reused record IDs"
Write-Host "[PASS] T003 persistent guest filesystem semantics"
Write-Host "[PASS] T003 write/truncate zero-extension behavior"
Write-Host "[PASS] T004 app_id/AID isolation boundary"
Write-Host "[PASS] T004 parent-traversal rejection"
Write-Host "[PASS] T005 RMS + file persistence across storage reopen"
Write-Host "[PASS] T005 app-scoped usage accounting"
Write-Host "[PASS] m32-storage -> m32-emulator-api workspace dependency boundary"
Write-Host "[PASS] previous 0.0.5 Audio version-close chain"
Write-Host "M32 0.0.6 Storage Bundle A verification passed."
exit 0
