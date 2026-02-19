# =============================================================================
# XKey Windows Uninstallation Script (PowerShell)
# =============================================================================
#
# This script removes the XKey Vietnamese Telex input method from Windows.
# It performs the following steps:
# 1. Stop any running xkey process
# 2. Remove from Windows startup (Registry Run key)
# 3. Delete installed files
#
# Usage:
#   .\windows\clean.ps1
#
# =============================================================================

$ErrorActionPreference = "Stop"

$InstallDir = "$env:LOCALAPPDATA\XKey"
$RegPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
$RegName = "XKey"

Write-Host "=== XKey Windows Uninstallation ===" -ForegroundColor Cyan
Write-Host ""

# Step 1: Stop running process
Write-Host "[1/3] Stopping xkey process..." -ForegroundColor Yellow
$proc = Get-Process -Name "xkey" -ErrorAction SilentlyContinue
if ($proc) {
    Stop-Process -Name "xkey" -Force
    Write-Host "Stopped xkey process."
} else {
    Write-Host "No running xkey process found."
}

# Step 2: Remove from startup
Write-Host "[2/3] Removing from Windows startup..." -ForegroundColor Yellow
$regValue = Get-ItemProperty -Path $RegPath -Name $RegName -ErrorAction SilentlyContinue
if ($regValue) {
    Remove-ItemProperty -Path $RegPath -Name $RegName
    Write-Host "Removed from startup."
} else {
    Write-Host "Not found in startup."
}

# Step 3: Delete installed files
Write-Host "[3/3] Removing installed files..." -ForegroundColor Yellow
if (Test-Path $InstallDir) {
    Remove-Item -Recurse -Force $InstallDir
    Write-Host "Removed $InstallDir"
} else {
    Write-Host "Install directory not found."
}

Write-Host ""
Write-Host "=== Uninstallation complete! ===" -ForegroundColor Green
Write-Host ""
