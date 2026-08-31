$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$CargoPath = Join-Path $RepoRoot "Cargo.toml"

Write-Host "M32 0.0.6 Storage Bundle A Workspace Apply"
Write-Host "=========================================="

if (-not (Test-Path -LiteralPath $CargoPath -PathType Leaf)) {
    throw "Root Cargo.toml not found: $CargoPath"
}

$Text = [System.IO.File]::ReadAllText($CargoPath)

if ($Text.Contains('"crates/m32-storage"')) {
    Write-Host "[PASS] m32-storage is already a workspace member"
    exit 0
}

$Newline = if ($Text.Contains("`r`n")) { "`r`n" } else { "`n" }
$Pattern = '(?m)^(\s*"crates/m32-library",\r?\n)'
$Regex = [regex]::new($Pattern)

if (-not $Regex.IsMatch($Text)) {
    throw 'Could not locate "crates/m32-library" workspace member anchor'
}

$Replacement = '${1}    "crates/m32-storage",' + $Newline
$Updated = $Regex.Replace($Text, $Replacement, 1)

[System.IO.File]::WriteAllText(
    $CargoPath,
    $Updated,
    [System.Text.UTF8Encoding]::new($false)
)

Write-Host "[PASS] added crates/m32-storage to root workspace"
