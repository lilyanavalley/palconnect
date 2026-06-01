# PalConnect init.d (SysV) Integration

This directory contains files for running PalConnect as a SysV-compatible init.d service
on **Linux systems that do not use systemd** (e.g. Alpine Linux with OpenRC, older Debian/Ubuntu
releases, or containers that use a minimal init process).

> **If your system uses systemd**, use the `systemd/` directory instead — it provides better
> integration and security hardening.

## Files

| File | Description |
|------|-------------|
| `palconnect` | init.d service script (`/etc/init.d/palconnect`) |
| `palconnect.env.example` | Environment configuration template |
| `install.sh` | Automated installation script |
| `uninstall.sh` | Automated uninstallation script |
| `README.md` | This documentation |

## Quick Installation

1. **Run the installation script** (as root):
   ```bash
   sudo ./initd/install.sh
   ```

2. **Configure your environment**:
   ```bash
   sudo nano /etc/palconnect/palconnect.env
   ```

3. **Start the service**:
   ```bash
   sudo /etc/init.d/palconnect start
   ```

## Manual Installation

### 1. Build the Application

```bash
cargo build --release
```

### 2. Create User and Directories

```bash
sudo useradd --system --shell /bin/false --home-dir /opt/palconnect --create-home palconnect
sudo mkdir -p /opt/palconnect /etc/palconnect /var/log/palconnect
```

### 3. Install Binary

```bash
sudo cp target/release/palconnect /opt/palconnect/
sudo chmod +x /opt/palconnect/palconnect
sudo chown -R palconnect:palconnect /opt/palconnect
```

### 4. Configure Environment

```bash
sudo cp initd/palconnect.env.example /etc/palconnect/palconnect.env
sudo chmod 600 /etc/palconnect/palconnect.env
sudo chown palconnect:palconnect /etc/palconnect/palconnect.env
sudo nano /etc/palconnect/palconnect.env
```

### 5. Install the init.d Script

```bash
sudo cp initd/palconnect /etc/init.d/palconnect
sudo chmod +x /etc/init.d/palconnect
```

### 6. Enable on Boot

**Debian / Ubuntu (update-rc.d)**:
```bash
sudo update-rc.d palconnect defaults
```

**RHEL / CentOS (chkconfig)**:
```bash
sudo chkconfig --add palconnect
sudo chkconfig palconnect on
```

**Alpine Linux / OpenRC**:
```bash
# OpenRC uses its own service framework; adapt the script or use an OpenRC unit directly.
sudo rc-update add palconnect default
```

## Configuration

Edit `/etc/palconnect/palconnect.env`:

```bash
# Required
DISCORD_TOKEN=your_discord_bot_token_here
PALWORLD_API_URL=http://localhost:8212
PALWORLD_ADMIN_PASSWORD=your_admin_password_here

# Service manager — must be "initd" on systems without systemd
PALWORLD_SERVICE_MANAGER=initd

# Name of the PalWorld init.d service script under /etc/init.d
PALWORLD_SERVICE_NAME=palworld
```

Setting `PALWORLD_SERVICE_MANAGER=initd` tells the bot's `/start` and `/forcestop`
Discord commands to manage the PalWorld server process using
`/etc/init.d/<PALWORLD_SERVICE_NAME> start|stop` and `kill -9` on the PID file,
rather than through `systemctl`.

## Service Management

```bash
# Start
sudo /etc/init.d/palconnect start

# Stop
sudo /etc/init.d/palconnect stop

# Restart
sudo /etc/init.d/palconnect restart

# Reload configuration (sends SIGHUP)
sudo /etc/init.d/palconnect reload

# Check status
sudo /etc/init.d/palconnect status
```

## Uninstallation

```bash
sudo ./initd/uninstall.sh
```

Or manually:

```bash
sudo /etc/init.d/palconnect stop
sudo update-rc.d -f palconnect remove   # Debian/Ubuntu
# sudo chkconfig --del palconnect       # RHEL/CentOS
sudo rm /etc/init.d/palconnect
# Optionally remove data:
sudo rm -rf /opt/palconnect /etc/palconnect /var/log/palconnect
sudo userdel palconnect
```
