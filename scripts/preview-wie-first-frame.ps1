$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$AdapterSource = Join-Path $RepoRoot "crates\m32-wie-adapter\src\lib.rs"
$PreviewDir = Join-Path $RepoRoot "target\m32-preview"
$Request = Join-Path $PreviewDir ".request-first-frame-bmp"
$Preview = Join-Path $PreviewDir "first-frame.bmp"

Write-Host "M32 Actual First-frame Preview"
Write-Host "=============================="

$SourceText = [System.IO.File]::ReadAllText($AdapterSource)
if (-not $SourceText.Contains(".request-first-frame-bmp")) {
    throw "FIX3 preview source marker is missing from m32-wie-adapter/src/lib.rs. Re-apply M32_0.0.3-T006_TickUntilFirstFrameHarness_FIX3_PREVIEW.zip before running this script."
}
if (-not $SourceText.Contains("write_first_frame_preview_bmp")) {
    throw "BMP preview exporter source is missing from m32-wie-adapter/src/lib.rs."
}

New-Item -ItemType Directory -Force -Path $PreviewDir | Out-Null

if (Test-Path -LiteralPath $Preview) {
    Remove-Item -LiteralPath $Preview -Force
}
if (Test-Path -LiteralPath $Request) {
    Remove-Item -LiteralPath $Request -Force
}

Set-Content -LiteralPath $Request -Value "M32 T006 actual first-frame preview request" -Encoding Ascii

try {
    Write-Host "[PASS] FIX3 preview source marker"
    Write-Host "[PASS] preview request marker created"
    Write-Host "Forcing m32-wie-adapter test binary rebuild..."

    cargo clean -p m32-wie-adapter
    if ($LASTEXITCODE -ne 0) {
        throw "cargo clean -p m32-wie-adapter failed with exit code $LASTEXITCODE"
    }

    cargo test -p m32-wie-adapter `
        tests::first_frame_paint_fixture_ticks_until_guest_frame_is_captured `
        -- --exact --nocapture

    if ($LASTEXITCODE -ne 0) {
        throw "Actual first-frame runtime test failed with exit code $LASTEXITCODE"
    }
}
finally {
    Remove-Item -LiteralPath $Request -Force -ErrorAction SilentlyContinue
}

if (-not (Test-Path -LiteralPath $Preview -PathType Leaf)) {
    throw "Runtime test passed but BMP preview was not written: $Preview"
}

$Info = Get-Item -LiteralPath $Preview

Write-Host ""
Write-Host "[PASS] actual J2ME guest frame captured"
Write-Host "[PASS] actual M32 RgbaFrame exported to 32-bit BMP"
Write-Host "[PASS] preview path: $Preview"
Write-Host "[PASS] preview bytes: $($Info.Length)"
Write-Host ""
Write-Host "Opening actual captured frame..."
Start-Process -FilePath $Preview

exit 0
