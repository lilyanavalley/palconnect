# PowerShell script to set up PalWorld as a Windows Service
# Run this as Administrator

param(
    [Parameter(Mandatory=$true)]
    [string]$PalServerPath,
    
    [string]$ServiceName = "PalWorldServer",
    [string]$ServiceDisplayName = "PalWorld Dedicated Server",
    [string]$ServiceDescription = "PalWorld game server for multiplayer gaming",
    [int]$Port = 8211,
    [int]$Players = 32
)

# Check if running as administrator
if (-NOT ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Error "This script must be run as Administrator"
    exit 1
}

# Verify PalServer.exe exists
if (-not (Test-Path $PalServerPath)) {
    Write-Error "PalServer.exe not found at: $PalServerPath"
    exit 1
}

Write-Host "Setting up PalWorld Windows Service..." -ForegroundColor Green

# Stop the service if it already exists
try {
    $existingService = Get-Service -Name $ServiceName -ErrorAction Stop
    if ($existingService.Status -eq 'Running') {
        Write-Host "Stopping existing service..." -ForegroundColor Yellow
        Stop-Service -Name $ServiceName -Force
    }
    Write-Host "Removing existing service..." -ForegroundColor Yellow
    & sc.exe delete $ServiceName
    Start-Sleep -Seconds 2
} catch {
    # Service doesn't exist, which is fine
}

# Create the service
$arguments = "-port=$Port -players=$Players"
Write-Host "Creating service with command: $PalServerPath $arguments" -ForegroundColor Cyan

& sc.exe create $ServiceName binPath= "`"$PalServerPath`" $arguments" DisplayName= $ServiceDisplayName start= demand
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to create service"
    exit 1
}

# Configure service description
& sc.exe description $ServiceName $ServiceDescription

# Configure service to restart on failure
& sc.exe failure $ServiceName reset= 86400 actions= restart/60000/restart/60000/restart/60000

Write-Host "Service created successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "Configure PalConnect with:" -ForegroundColor Cyan
Write-Host "[server_management]" -ForegroundColor White
Write-Host "service_type = `"windowsservice`"" -ForegroundColor White
Write-Host "service_name = `"$ServiceName`"" -ForegroundColor White
Write-Host ""
Write-Host "You can start the service with: Start-Service -Name '$ServiceName'" -ForegroundColor Yellow
Write-Host "Or use: sc start $ServiceName" -ForegroundColor Yellow
