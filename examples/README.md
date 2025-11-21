# PalConnect Service Management Examples

This directory contains example setup scripts and configuration files for different platforms and service management systems.

## Files Overview

### Linux (systemd)
- `setup-systemd.sh` - Setup script for systemd service
- Configure with: `service_type = "systemd"`

### macOS (launchd)  
- `setup-launchd.sh` - Setup script for launchd service
- `com.palworld.server.plist` - Example launchd plist file
- Configure with: `service_type = "launchd"`

### Windows
- `setup-windows-service.ps1` - PowerShell script to create Windows Service
- Configure with: `service_type = "windowsservice"`

### Docker (Custom Script)
- `docker-server-control.sh` - Example Docker container management script
- Configure with: `service_type = "customscript"`

## Usage Instructions

### Linux (systemd) Setup

1. **Edit the script** (optional):
   ```bash
   nano examples/setup-systemd.sh
   # Modify PALWORLD_USER, SERVER_DIR, etc. as needed
   ```

2. **Run the setup script**:
   ```bash
   sudo ./examples/setup-systemd.sh
   ```

3. **Configure PalConnect** in `Config.toml`:
   ```toml
   [server_management]
   service_type = "systemd"
   service_name = "palworld.service"
   ```

### macOS (launchd) Setup

1. **Edit the plist file** (required):
   ```bash
   nano examples/com.palworld.server.plist
   # Update ProgramArguments, WorkingDirectory paths
   ```

2. **Run the setup script**:
   ```bash
   sudo ./examples/setup-launchd.sh
   ```

3. **Load the service**:
   ```bash
   sudo launchctl load -w /Library/LaunchDaemons/com.palworld.server.plist
   ```

4. **Configure PalConnect** in `Config.toml`:
   ```toml
   [server_management]
   service_type = "launchd"
   service_name = "/Library/LaunchDaemons/com.palworld.server.plist"
   ```

### Windows Service Setup

1. **Open PowerShell as Administrator**

2. **Run the setup script**:
   ```powershell
   .\examples\setup-windows-service.ps1 -PalServerPath "C:\PalWorld\PalServer.exe"
   ```

3. **Configure PalConnect** in `Config.toml`:
   ```toml
   [server_management]
   service_type = "windowsservice"
   service_name = "PalWorldServer"
   ```

### Docker Setup

1. **Edit the Docker script** (required):
   ```bash
   nano examples/docker-server-control.sh
   # Update IMAGE_NAME, DATA_DIR, ports, etc.
   ```

2. **Make sure Docker is installed and running**

3. **Test the script**:
   ```bash
   # Test starting the server
   ./examples/docker-server-control.sh start
   
   # Test stopping the server
   ./examples/docker-server-control.sh stop
   ```

4. **Configure PalConnect** in `Config.toml`:
   ```toml
   [server_management]
   service_type = "customscript"
   start_command = "/path/to/examples/docker-server-control.sh start"
   stop_command = "/path/to/examples/docker-server-control.sh stop"
   force_stop_command = "/path/to/examples/docker-server-control.sh force-stop"
   ```

## Custom Script Examples

### Simple Process Management
```toml
[server_management]
service_type = "customscript"
start_command = "/usr/local/bin/palworld-start.sh"
stop_command = "/usr/local/bin/palworld-stop.sh"
force_stop_command = "pkill -9 PalServer"
```

### Windows PowerShell
```toml
[server_management]
service_type = "powershell"
start_command = "Start-Process -FilePath 'C:\\PalWorld\\PalServer.exe' -ArgumentList '-port=8211' -WindowStyle Hidden"
stop_command = "Stop-Process -Name 'PalServer' -Force"
force_stop_command = "Stop-Process -Name 'PalServer' -Force"
```

### Process Manager (PM2, supervisord, etc.)
```toml
[server_management]
service_type = "customscript"
start_command = "pm2 start palworld"
stop_command = "pm2 stop palworld"
force_stop_command = "pm2 kill palworld"
```

## Security Notes

### Permissions Required

- **Linux**: PalConnect needs permission to control systemd services via `systemctl`
- **macOS**: PalConnect typically needs to run with elevated privileges for launchd
- **Windows**: Administrator privileges required for service management
- **Custom Scripts**: Ensure scripts are executable and PalConnect has permission to run them

### Recommended Security Setup

For production deployments, consider:

1. **Create dedicated user** for PalConnect
2. **Use sudo rules** to limit permissions to specific commands
3. **Set proper file permissions** on scripts and configuration files
4. **Run services as non-root users** when possible

Example sudo rule for Linux:
```bash
# /etc/sudoers.d/palconnect
palconnect ALL=(root) NOPASSWD: /bin/systemctl start palworld.service, /bin/systemctl stop palworld.service, /bin/systemctl kill palworld.service
```

## Troubleshooting

### Common Issues

1. **Permission Denied**: Ensure PalConnect has necessary permissions
2. **Service Not Found**: Verify service names and paths in configuration
3. **Command Not Found**: Check that required tools are installed and in PATH
4. **Timeout Errors**: Some operations may take time, especially on slower systems

### Testing Configuration

Test your setup manually before configuring PalConnect:

```bash
# Linux systemd
sudo systemctl start palworld.service
sudo systemctl stop palworld.service

# macOS launchd  
sudo launchctl load -w /Library/LaunchDaemons/com.palworld.server.plist
sudo launchctl unload -w /Library/LaunchDaemons/com.palworld.server.plist

# Windows (as Administrator)
sc start PalWorldServer
sc stop PalWorldServer

# Custom scripts
./docker-server-control.sh start
./docker-server-control.sh stop
```

### Logs and Debugging

- Check PalConnect logs for service management errors
- Check system logs (journalctl, Event Viewer, Console.app) for service issues
- Verify PalWorld server logs for application-specific problems

For more detailed information, see the main documentation at `docs/SERVICE_MANAGEMENT.md`.
