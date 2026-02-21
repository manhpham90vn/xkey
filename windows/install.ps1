# =============================================================================
# XKey Windows Installation Script (PowerShell)
# =============================================================================
#
# This script installs the XKey Vietnamese Telex input method on Windows.
# It performs the following steps:
# 1. Build the project in release mode
# 2. Copy the binary to %LOCALAPPDATA%\XKey\
#
# Prerequisites:
# - Rust toolchain (for building)
#
# Usage:
#   .\windows\install.ps1
#
# =============================================================================

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectDir = Split-Path -Parent $ScriptDir
$InstallDir = "$env:LOCALAPPDATA\XKey"
$ExePath = "$InstallDir\xkey.exe"

Write-Host "=== XKey Windows Installation ===" -ForegroundColor Cyan
Write-Host ""

# Step 1: Build the project in release mode
Write-Host "[1/2] Building xkey in release mode..." -ForegroundColor Yellow
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "Rust is not installed. Please install it from https://rustup.rs/"
    exit 1
}
Push-Location $ProjectDir
try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Build failed!"
        exit 1
    }
} finally {
    Pop-Location
}

# Step 2: Install the binary
Write-Host "[2/2] Installing binary to $InstallDir..." -ForegroundColor Yellow
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}
Copy-Item "$ProjectDir\target\release\xkey.exe" $ExePath -Force

Write-Host ""
Write-Host "=== Installation complete! ===" -ForegroundColor Green
Write-Host ""
Write-Host "XKey has been installed to $InstallDir."
Write-Host "To start now, run: $ExePath"
Write-Host "To stop, press Ctrl+C in the terminal or close the process."
Write-Host ""
