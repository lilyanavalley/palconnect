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
# PalConnect Systemd Uninstallation Script
# This script removes PalConnect systemd service and related files
#######################################################################################################################

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   print_error "This script must be run as root (use sudo)"
   exit 1
fi

print_status "Starting PalConnect systemd uninstallation..."

# Stop and disable service
if systemctl is-active --quiet palconnect.service; then
    print_status "Stopping PalConnect service..."
    systemctl stop palconnect.service
    print_success "Service stopped"
fi

if systemctl is-enabled --quiet palconnect.service; then
    print_status "Disabling PalConnect service..."
    systemctl disable palconnect.service
    print_success "Service disabled"
fi

# Remove systemd service file
if [ -f /etc/systemd/system/palconnect.service ]; then
    print_status "Removing systemd service file..."
    rm /etc/systemd/system/palconnect.service
    systemctl daemon-reload
    print_success "Systemd service file removed"
fi

# Ask about removing files
read -p "Remove application files from /opt/palconnect? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_status "Removing application files..."
    rm -rf /opt/palconnect
    print_success "Application files removed"
fi

read -p "Remove configuration files from /etc/palconnect? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_status "Removing configuration files..."
    rm -rf /etc/palconnect
    print_success "Configuration files removed"
fi

read -p "Remove log files from /var/log/palconnect? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_status "Removing log files..."
    rm -rf /var/log/palconnect
    print_success "Log files removed"
fi

read -p "Remove palconnect user account? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_status "Removing palconnect user..."
    userdel palconnect 2>/dev/null || print_warning "User removal failed (may not exist)"
    print_success "User removed"
fi

print_success "Uninstallation completed!"
