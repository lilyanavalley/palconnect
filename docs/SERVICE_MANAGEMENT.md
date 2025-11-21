# Cross-Platform Service Management

PalConnect now supports starting and stopping PalWorld servers across multiple platforms and service management systems. This document explains how to configure each option.

## Configuration

Add the following section to your `Config.toml` file:

```toml
[server_management]
service_type = "systemd"  # Choose your platform
service_name = "palworld.service"
```

## Platform Options

### Linux with systemd (Default)

Most modern Linux distributions use systemd.

```toml
[server_management]
service_type = "systemd"
service_name = "palworld.service"
```

Requirements:
- A systemd service file for your PalWorld server
- PalConnect must run with sufficient permissions to control the service

### macOS with launchd

macOS uses launchd for service management.

```toml
[server_management]
service_type = "launchd"
service_name = "/Library/LaunchDaemons/com.palworld.server.plist"
```

Requirements:
- A launchd plist file for your PalWorld server
- PalConnect must run with sufficient permissions (usually requires sudo)

Example plist file (`/Library/LaunchDaemons/com.palworld.server.plist`):
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.palworld.server</string>
    <key>ProgramArguments</key>
    <array>
        <string>/path/to/PalServer</string>
        <string>-port=8211</string>
        <string>-players=32</string>
    </array>
    <key>KeepAlive</key>
    <true/>
    <key>RunAtLoad</key>
    <true/>
    <key>UserName</key>
    <string>palworld</string>
    <key>WorkingDirectory</key>
    <string>/path/to/palworld</string>
</dict>
</plist>
```

### Windows Services

For PalWorld servers installed as Windows Services.

```toml
[server_management]
service_type = "windowsservice"
service_name = "PalWorldServer"
```

Requirements:
- PalWorld server installed as a Windows Service
- PalConnect must run with Administrator privileges

### Windows PowerShell Scripts

For more complex Windows setups or custom PowerShell scripts.

```toml
[server_management]
service_type = "powershell"
start_command = "Start-Service -Name 'PalWorldServer'"
stop_command = "Stop-Service -Name 'PalWorldServer'"
force_stop_command = "Stop-Process -Name 'PalServer' -Force"
```

Alternative example with custom scripts:
```toml
[server_management]
service_type = "powershell"
start_command = "& 'C:\\PalWorld\\start-server.ps1'"
stop_command = "& 'C:\\PalWorld\\stop-server.ps1'"
force_stop_command = "Stop-Process -Name 'PalServer' -Force"
```

### Custom Scripts (Universal)

For any platform with custom start/stop scripts.

```toml
[server_management]
service_type = "customscript"
start_command = "/usr/local/bin/start-palworld.sh"
stop_command = "/usr/local/bin/stop-palworld.sh"
force_stop_command = "/usr/local/bin/force-stop-palworld.sh"
```

Docker example:
```toml
[server_management]
service_type = "customscript"
start_command = "docker start palworld-container"
stop_command = "docker stop palworld-container"
force_stop_command = "docker kill palworld-container"
```

## Security Considerations

### Linux (systemd)
- PalConnect needs permission to control systemd services
- Consider using sudo rules to limit permissions:
  ```bash
  # Add to /etc/sudoers.d/palconnect
  palconnect ALL=(root) NOPASSWD: /bin/systemctl start palworld.service, /bin/systemctl stop palworld.service, /bin/systemctl kill palworld.service
  ```

### macOS (launchd)
- PalConnect typically needs to run with elevated privileges
- Consider using sudo rules similar to Linux

### Windows
- PalConnect needs Administrator privileges for service management
- For PowerShell scripts, ensure execution policy allows script execution:
  ```powershell
  Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope LocalMachine
  ```

## Troubleshooting

### Permission Errors
- Ensure PalConnect has the necessary permissions for your chosen service type
- Check system logs for detailed error messages

### Service Not Found
- Verify the service name/path is correct
- Ensure the service is properly installed and configured

### Command Not Found
- For custom scripts, verify the script paths are correct and executable
- Check that required tools (docker, etc.) are installed and in PATH

### Timeouts
- Some service operations may take time to complete
- PalConnect includes reasonable timeouts, but very slow systems may need adjustments

## Example Service Files

### systemd Service File
Create `/etc/systemd/system/palworld.service`:
```ini
[Unit]
Description=PalWorld Dedicated Server
After=network.target

[Service]
Type=simple
User=palworld
Group=palworld
WorkingDirectory=/home/palworld/server
ExecStart=/home/palworld/server/PalServer -port=8211 -players=32
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Then enable it:
```bash
sudo systemctl enable palworld.service
sudo systemctl daemon-reload
```

### Windows PowerShell Scripts

Example `start-server.ps1`:
```powershell
# Start PalWorld Server
$processName = "PalServer"
$serverPath = "C:\PalWorld\PalServer.exe"
$arguments = "-port=8211 -players=32"

if (Get-Process $processName -ErrorAction SilentlyContinue) {
    Write-Host "Server is already running"
    exit 0
}

Start-Process -FilePath $serverPath -ArgumentList $arguments -WindowStyle Hidden
Write-Host "Server started successfully"
```

Example `stop-server.ps1`:
```powershell
# Stop PalWorld Server gracefully
$processName = "PalServer"

$process = Get-Process $processName -ErrorAction SilentlyContinue
if ($process) {
    $process.CloseMainWindow()
    $process.WaitForExit(30000)  # Wait 30 seconds
    if (!$process.HasExited) {
        $process.Kill()
    }
    Write-Host "Server stopped successfully"
} else {
    Write-Host "Server is not running"
}
```

## Testing Your Configuration

After configuring your service management, you can test it using Discord slash commands:

- `/start` - Start the PalWorld server
- `/stop` - Gracefully stop the server (with optional delay and message)
- `/forcestop` - Force stop the server immediately

Monitor the PalConnect logs for any errors during service operations.
