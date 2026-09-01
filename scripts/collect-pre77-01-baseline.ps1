param(
    [switch]$RequireClean,
    [switch]$SkipVersionClose
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$outDir = Join-Path $RepoRoot 'target\pre77'
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$logPath = Join-Path $outDir "PRE77-01-baseline-$stamp.log"

function Write-Section([string]$Title) {
    Write-Host ''
    Write-Host ('=' * 72)
    Write-Host $Title
    Write-Host ('=' * 72)
}

function Invoke-Native([string]$Label, [scriptblock]$Command) {
    Write-Host "[RUN] $Label"
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed with exit code $LASTEXITCODE"
    }
    Write-Host "[PASS] $Label"
}

Start-Transcript -Path $logPath -Force | Out-Null
try {
    Write-Host 'M32 PRE77-01 Clean Reproducible Baseline Probe'
    Write-Host '================================================'
    Write-Host "Repository: $RepoRoot"
    Write-Host "Log: $logPath"

    Write-Section 'A. Repository identity'
    $head = (& git rev-parse HEAD | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) {
        throw "git rev-parse HEAD failed with exit code $LASTEXITCODE"
    }

    $branch = (& git branch --show-current | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) {
        throw "git branch --show-current failed with exit code $LASTEXITCODE"
    }

    $status = (& git status --short | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) {
        throw "git status --short failed with exit code $LASTEXITCODE"
    }
    $lockHash = (Get-FileHash (Join-Path $RepoRoot 'Cargo.lock') -Algorithm SHA256).Hash.ToLowerInvariant()

    Write-Host "HEAD: $head"
    Write-Host "Branch: $branch"
    Write-Host "Cargo.lock SHA-256: $lockHash"
    Write-Host 'Working tree:'
    if ([string]::IsNullOrWhiteSpace($status)) {
        Write-Host '  CLEAN'
    } else {
        $status -split "`r?`n" | ForEach-Object { Write-Host "  $_" }
    }

    if ($RequireClean -and -not [string]::IsNullOrWhiteSpace($status)) {
        throw 'PRE77-01 clean gate requires an empty git status. Commit or intentionally remove current changes first.'
    }

    Write-Section 'B. Toolchain identity'
    Invoke-Native 'rustc --version --verbose' { rustc --version --verbose }
    Invoke-Native 'cargo --version --verbose' { cargo --version --verbose }

    Write-Section 'C. Locked dependency graph'
    $metadataJson = & cargo metadata --locked --format-version 1
    if ($LASTEXITCODE -ne 0) {
        throw "cargo metadata --locked failed with exit code $LASTEXITCODE"
    }
    $metadata = $metadataJson | ConvertFrom-Json
    $gitPackages = @($metadata.packages | Where-Object { $_.source -and $_.source.StartsWith('git+') })
    if ($gitPackages.Count -eq 0) {
        Write-Host '[INFO] No git-sourced packages in cargo metadata.'
    } else {
        foreach ($pkg in $gitPackages | Sort-Object name, version) {
            Write-Host ("[GIT DEP] {0} {1} :: {2}" -f $pkg.name, $pkg.version, $pkg.source)
        }
    }

    Write-Section 'D. Cargo git checkout contamination check'
    $checkoutRoot = Join-Path $HOME '.cargo\git\checkouts'
    $dirtyCheckout = $false
    $matchedCheckout = $false
    if (Test-Path $checkoutRoot) {
        $repos = Get-ChildItem -Path $checkoutRoot -Directory -ErrorAction SilentlyContinue |
            ForEach-Object { Get-ChildItem -Path $_.FullName -Directory -ErrorAction SilentlyContinue }

        foreach ($repo in $repos) {
            if (-not (Test-Path (Join-Path $repo.FullName '.git'))) { continue }
            $origin = (& git -C $repo.FullName remote get-url origin 2>$null | Out-String).Trim()
            if ($LASTEXITCODE -ne 0) { continue }
            if ($origin -notmatch '(?i)(/wie(?:\.git)?$|/RustJava(?:\.git)?$|/smaf(?:\.git)?$)') { continue }

            $matchedCheckout = $true
            $checkoutHead = (& git -C $repo.FullName rev-parse HEAD | Out-String).Trim()
            $checkoutStatus = (& git -C $repo.FullName status --porcelain=v1 --untracked-files=all | Out-String).Trim()
            Write-Host "Checkout: $($repo.FullName)"
            Write-Host "  origin: $origin"
            Write-Host "  HEAD:   $checkoutHead"
            if ([string]::IsNullOrWhiteSpace($checkoutStatus)) {
                Write-Host '  status: CLEAN'
            } else {
                $dirtyCheckout = $true
                Write-Host '  status: DIRTY'
                $checkoutStatus -split "`r?`n" | ForEach-Object { Write-Host "    $_" }
            }
        }
    }

    if (-not $matchedCheckout) {
        Write-Host '[INFO] No local Cargo checkout for WIE/RustJava/SMAF was found. cargo metadata above remains the source-of-truth dependency record.'
    }
    if ($dirtyCheckout) {
        throw 'A WIE/RustJava/SMAF Cargo git checkout contains local modifications. PRE77-01 forbids checkout patching.'
    }
    Write-Host '[PASS] relevant Cargo git checkouts contain no local modifications'

    Write-Section 'E. Existing 0.1.0 canonical verification'
    $versionClose = Join-Path $RepoRoot 'scripts\verify-first-playable-version-close.ps1'
    if ($SkipVersionClose) {
        Write-Host '[SKIP] version-close verifier skipped by caller'
    } elseif (Test-Path $versionClose) {
        Invoke-Native '0.1.0 First Playable version-close verifier' {
            powershell -ExecutionPolicy Bypass -File $versionClose
        }
    } else {
        throw "Missing canonical verifier: $versionClose"
    }

    Write-Section 'F. Two consecutive locked desktop builds'
    Invoke-Native 'desktop locked build #1' { cargo build -p m32-desktop --locked }

    $targetDir = [string]$metadata.target_directory
    $candidateExe = @(
        (Join-Path $targetDir 'x86_64-pc-windows-msvc\debug\m32.exe'),
        (Join-Path $targetDir 'debug\m32.exe')
    ) | Where-Object { Test-Path $_ } | Select-Object -First 1

    if (-not $candidateExe) {
        throw "Could not locate m32.exe under Cargo target directory: $targetDir"
    }
    $hash1 = (Get-FileHash $candidateExe -Algorithm SHA256).Hash.ToLowerInvariant()
    Write-Host "Build #1 m32.exe: $candidateExe"
    Write-Host "Build #1 SHA-256: $hash1"

    Invoke-Native 'desktop locked build #2' { cargo build -p m32-desktop --locked }
    $hash2 = (Get-FileHash $candidateExe -Algorithm SHA256).Hash.ToLowerInvariant()
    Write-Host "Build #2 SHA-256: $hash2"

    if ($hash1 -ne $hash2) {
        throw "Consecutive unchanged builds produced different m32.exe hashes: $hash1 != $hash2"
    }
    Write-Host '[PASS] consecutive unchanged locked builds retain identical m32.exe SHA-256'

    Write-Section 'G. PRE77-01 probe result'
    Write-Host "[PASS] repository identity captured at $head"
    Write-Host "[PASS] Cargo.lock identity captured at $lockHash"
    Write-Host '[PASS] locked dependency graph resolves'
    Write-Host '[PASS] WIE/RustJava/SMAF Cargo checkout contamination = 0'
    Write-Host '[PASS] two consecutive locked desktop builds succeeded with stable executable hash'
    if ($RequireClean) {
        Write-Host '[PASS] repository working tree is clean'
        Write-Host 'PRE77-01 CLEAN BASELINE GATE PASSED.'
    } else {
        Write-Host '[INFO] Probe mode complete. This does not close PRE77-01 until the same script passes with -RequireClean.'
    }
}
finally {
    Stop-Transcript | Out-Null
}
