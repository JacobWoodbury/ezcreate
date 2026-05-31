# ezcreate - first-time setup (Windows)
# Run from repo root:  .\scripts\setup.ps1
# Or:                  powershell -ExecutionPolicy Bypass -File .\scripts\setup.ps1

$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

Write-Host ""
Write-Host "ezcreate - first-time setup (Windows)" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

function Test-MsvcLinker {
    $link = Get-Command link.exe -ErrorAction SilentlyContinue
    if ($link) {
        return $true
    }
    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vswhere) {
        $installPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
        if ($installPath) {
            $found = Get-ChildItem -Path "$installPath\VC\Tools\MSVC\*\bin\Hostx64\x64\link.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
            return $null -ne $found
        }
    }
    return $false
}

# --- 1. Rust ---
Write-Host "[1/3] Checking Rust..." -ForegroundColor Yellow
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host "  Rust is not installed." -ForegroundColor Red
    Write-Host "  Install rustup: https://rustup.rs" -ForegroundColor White
    Write-Host "  Then in a new terminal run:" -ForegroundColor White
    Write-Host "    rustup default stable-x86_64-pc-windows-msvc" -ForegroundColor Gray
    exit 1
}

$rustHost = (rustc -vV | Select-String "^host: (.+)$").Matches.Groups[1].Value
Write-Host "  rustc: $(rustc --version)" -ForegroundColor Green
Write-Host "  host:  $rustHost" -ForegroundColor Green

if ($rustHost -notmatch "windows-msvc") {
    Write-Host "  Warning: MSVC toolchain recommended on Windows." -ForegroundColor Yellow
    Write-Host "  Run: rustup default stable-x86_64-pc-windows-msvc" -ForegroundColor Gray
}

# --- 2. MSVC Build Tools (link.exe) ---
Write-Host ""
Write-Host "[2/3] Checking MSVC linker (link.exe)..." -ForegroundColor Yellow
if (Test-MsvcLinker) {
    Write-Host "  MSVC C++ tools found." -ForegroundColor Green
} else {
    Write-Host "  link.exe not found - Rust cannot build this project yet." -ForegroundColor Red
    Write-Host ""
    Write-Host "  Install Visual Studio Build Tools 2022 with C++:" -ForegroundColor White
    Write-Host "    https://visualstudio.microsoft.com/visual-cpp-build-tools/" -ForegroundColor Gray
    Write-Host ""
    Write-Host "  Select workload: Desktop development with C++" -ForegroundColor White
    Write-Host "  Or run (Admin PowerShell) from this folder:" -ForegroundColor White
    Write-Host '    .\vs_buildtools.exe --passive --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended' -ForegroundColor Gray
    Write-Host ""
    Write-Host "  After install, open a NEW terminal and run this script again." -ForegroundColor White
    exit 1
}

# --- 3. Build check ---
Write-Host ""
Write-Host "[3/3] Running cargo check (first time may take 10-30+ min)..." -ForegroundColor Yellow
Write-Host ""

cargo check
if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "Build check failed. See errors above." -ForegroundColor Red
    Write-Host "Tips: use PowerShell (not Git Bash) for cargo, and see docs/GETTING_STARTED.md" -ForegroundColor Gray
    exit $LASTEXITCODE
}

Write-Host ""
Write-Host "Setup complete. Run the game with:" -ForegroundColor Green
Write-Host "  cargo run" -ForegroundColor White
Write-Host "  .\scripts\run.ps1" -ForegroundColor Gray
Write-Host ""
Write-Host "Walkthrough: docs/GETTING_STARTED.md" -ForegroundColor Cyan
Write-Host ""
