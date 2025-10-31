# PalConnect Systemd Integration

This directory contains files for running PalConnect as a systemd service on Linux systems.

## Files

- `palconnect.service` - Systemd unit file
- `palconnect.env.example` - Environment configuration template
- `install.sh` - Automated installation script
- `uninstall.sh` - Automated uninstallation script
- `README.md` - This documentation

## Quick Installation

1. **Run the installation script**:
   ```bash
   sudo ./systemd/install.sh
   ```

2. **Configure your environment**:
   ```bash
   sudo nano /etc/palconnect/palconnect.env
   ```

3. **Start the service**:
   ```bash
   sudo systemctl start palconnect
   ```

## Manual Installation

### 1. Build the Application

```bash
cargo build --release
```

### 2. Create User and Directories

```bash
# Create system user
sudo useradd --system --shell /bin/false --home-dir /opt/palconnect --create-home palconnect

# Create directories
sudo mkdir -p /opt/palconnect
sudo mkdir -p /etc/palconnect
sudo mkdir -p /var/log/palconnect
```

### 3. Install Binary

```bash
sudo cp target/release/palconnect /opt/palconnect/
sudo chmod +x /opt/palconnect/palconnect
sudo chown palconnect:palconnect /opt/palconnect/palconnect
```

### 4. Configure Environment

```bash
sudo cp systemd/palconnect.env.example /etc/palconnect/palconnect.env
sudo chmod 600 /etc/palconnect/palconnect.env
sudo chown palconnect:palconnect /etc/palconnect/palconnect.env

# Edit configuration
sudo nano /etc/palconnect/palconnect.env
```

### 5. Install Systemd Service

```bash
sudo cp systemd/palconnect.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable palconnect.service
```

### 6. Set Permissions

```bash
sudo chown -R palconnect:palconnect /opt/palconnect
sudo chown palconnect:palconnect /var/log/palconnect
sudo chmod 755 /var/log/palconnect
```

## Configuration

### Environment Variables

Edit `/etc/palconnect/palconnect.env`:

```bash
# Required: Discord Bot Token
DISCORD_TOKEN=your_discord_bot_token_here

# Required: PalWorld Server Configuration
PALWORLD_API_URL=http://localhost:8212
PALWORLD_ADMIN_PASSWORD=your_admin_password_here

# Optional: Auto-update (default: false)
UPDATES_AUTO_ENABLE=false

# Optional: Logging level (default: info)
RUST_LOG=info
```

## Service Management

### Start/Stop/Restart

```bash
# Start the service
sudo systemctl start palconnect

# Stop the service
sudo systemctl stop palconnect

# Restart the service
sudo systemctl restart palconnect

# Reload configuration (sends SIGHUP)
sudo systemctl reload palconnect
```

### Status and Logs

```bash
# Check service status
sudo systemctl status palconnect

# View logs (follow mode)
sudo journalctl -u palconnect -f

# View recent logs
sudo journalctl -u palconnect --since "1 hour ago"

# View all logs
sudo journalctl -u palconnect
```

### Enable/Disable Auto-start

```bash
# Enable auto-start on boot
sudo systemctl enable palconnect

# Disable auto-start on boot
sudo systemctl disable palconnect
```

## Security Features

The systemd service includes comprehensive security hardening:

- **Process Isolation**: Runs as dedicated `palconnect` user
- **Filesystem Protection**: Read-only root filesystem, limited write access
- **Network Security**: Standard network access only
- **Resource Limits**: CPU and memory limits
- **Privilege Restrictions**: No new privileges, no SUID/SGID
- **Kernel Protection**: Restricted kernel access

## Monitoring

### Health Check Endpoint

The service exposes a health check endpoint at `http://localhost:3000/health` for monitoring systems.

### Log Monitoring

Monitor these log messages for service health:

```bash
# Service started successfully
grep "Starting both Discord bot and health check server" /var/log/syslog

# Service errors
sudo journalctl -u palconnect -p err

# Discord connection status
sudo journalctl -u palconnect | grep "Discord"
```

## Troubleshooting

### Common Issues

1. **Service fails to start**:
   ```bash
   # Check configuration
   sudo systemctl status palconnect
   sudo journalctl -u palconnect --no-pager
   ```

2. **Permission denied errors**:
   ```bash
   # Fix ownership
   sudo chown -R palconnect:palconnect /opt/palconnect
   ```

3. **Discord connection issues**:
   ```bash
   # Verify token in environment file
   sudo cat /etc/palconnect/palconnect.env | grep DISCORD_TOKEN
   ```

4. **PalWorld server connection issues**:
   ```bash
   # Test API endpoint
   curl -u admin:your_password http://localhost:8212/v1/api/info
   ```

### Log Levels

Adjust logging verbosity by setting `RUST_LOG` in `/etc/palconnect/palconnect.env`:

- `error` - Errors only
- `warn` - Warnings and errors
- `info` - Info, warnings, and errors (default)
- `debug` - Debug info and above
- `trace` - All messages (very verbose)

## Uninstallation

Run the uninstall script:

```bash
sudo ./systemd/uninstall.sh
```

Or manually:

```bash
# Stop and disable service
sudo systemctl stop palconnect
sudo systemctl disable palconnect

# Remove service file
sudo rm /etc/systemd/system/palconnect.service
sudo systemctl daemon-reload

# Remove files (optional)
sudo rm -rf /opt/palconnect
sudo rm -rf /etc/palconnect
sudo rm -rf /var/log/palconnect

# Remove user (optional)
sudo userdel palconnect
```

## Integration with Monitoring Systems

### Prometheus/Grafana

Monitor via the health endpoint:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'palconnect'
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: '/health'
```

### Nagios/Icinga

```bash
# Check command
define command {
    command_name    check_palconnect
    command_line    /usr/lib/nagios/plugins/check_http -H localhost -p 3000 -u /health
}
```

### systemd Watchdog

The service is configured with automatic restart on failure and includes timeout settings for reliable operation.
