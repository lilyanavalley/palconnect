#!/bin/bash

# 
# PalConnect - A Discord bot for PalWorld server monitoring
# Copyright (C) 2025  Lily Ana Valley <hi@lilyvalley.dev> <https://lilyvalley.dev>
# 
# This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General 
# Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) 
# any later version.
# 
# This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied 
# warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more
# details.
# 
# You should have received a copy of the GNU Affero General Public License along with this program.  If not, see
# <https://www.gnu.org/licenses/>.
# 
#######################################################################################################################
# PalConnect init.d (SysV) Installation Script
# Installs PalConnect as a SysV-compatible init.d service (for Linux systems without systemd)
#######################################################################################################################

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status()  { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error()   { echo -e "${RED}[ERROR]${NC} $1"; }

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   print_error "This script must be run as root (use sudo)"
   exit 1
fi

# Refuse to run on a systemd-managed system to avoid conflicts
if pidof systemd > /dev/null 2>&1; then
    print_warning "systemd is running on this system."
    print_warning "Consider using the systemd/ installer instead for better integration."
    read -r -p "Continue with init.d installation anyway? [y/N] " REPLY
    [[ "$REPLY" =~ ^[Yy]$ ]] || { echo "Aborted."; exit 1; }
fi

print_status "Starting PalConnect init.d installation..."

# Create palconnect user and group
print_status "Creating palconnect user and group..."
if ! id "palconnect" &>/dev/null; then
    useradd --system --shell /bin/false --home-dir /opt/palconnect --create-home palconnect
    print_success "Created palconnect user"
else
    print_warning "palconnect user already exists"
fi

# Create directories
print_status "Creating directories..."
mkdir -p /opt/palconnect
mkdir -p /etc/palconnect
mkdir -p /var/log/palconnect

# Build the application (if not already built)
if [ ! -f "$PROJECT_ROOT/target/release/palconnect" ]; then
    print_status "Building PalConnect application..."
    cd "$PROJECT_ROOT"
    cargo build --release
    print_success "Application built successfully"
fi

# Copy binary
print_status "Installing binary..."
cp "$PROJECT_ROOT/target/release/palconnect" /opt/palconnect/
chmod +x /opt/palconnect/palconnect
chown palconnect:palconnect /opt/palconnect/palconnect
print_success "Binary installed to /opt/palconnect/palconnect"

# Copy environment file template
print_status "Installing environment configuration..."
if [ ! -f /etc/palconnect/palconnect.env ]; then
    cp "$SCRIPT_DIR/palconnect.env.example" /etc/palconnect/palconnect.env
    chmod 600 /etc/palconnect/palconnect.env
    chown palconnect:palconnect /etc/palconnect/palconnect.env
    print_success "Environment file installed to /etc/palconnect/palconnect.env"
    print_warning "Please edit /etc/palconnect/palconnect.env with your configuration"
else
    print_warning "Environment file already exists at /etc/palconnect/palconnect.env"
fi

# Set up log directory
chown palconnect:palconnect /var/log/palconnect
chmod 755 /var/log/palconnect

# Set ownership
chown -R palconnect:palconnect /opt/palconnect

# Install init.d script
print_status "Installing init.d service script..."
cp "$SCRIPT_DIR/palconnect" /etc/init.d/palconnect
chmod +x /etc/init.d/palconnect
print_success "init.d script installed to /etc/init.d/palconnect"

# Enable on boot using available tooling
print_status "Enabling PalConnect service on boot..."
if command -v update-rc.d &>/dev/null; then
    update-rc.d palconnect defaults
    print_success "Enabled via update-rc.d"
elif command -v chkconfig &>/dev/null; then
    chkconfig --add palconnect
    chkconfig palconnect on
    print_success "Enabled via chkconfig"
else
    print_warning "Could not detect update-rc.d or chkconfig — please enable the service manually."
fi

print_success "Installation completed successfully!"
echo
print_status "Next steps:"
echo "1. Edit /etc/palconnect/palconnect.env with your configuration"
echo "2. Start the service:  sudo /etc/init.d/palconnect start"
echo "3. Check status:       sudo /etc/init.d/palconnect status"
echo "4. View logs:          tail -f /var/log/palconnect/palconnect.log"
echo
print_warning "Don't forget to configure your Discord token and PalWorld server settings!"
