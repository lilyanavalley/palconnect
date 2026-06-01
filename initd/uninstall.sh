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
# PalConnect init.d (SysV) Uninstallation Script
#######################################################################################################################

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status()  { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error()   { echo -e "${RED}[ERROR]${NC} $1"; }

if [[ $EUID -ne 0 ]]; then
   print_error "This script must be run as root (use sudo)"
   exit 1
fi

print_status "Starting PalConnect init.d uninstallation..."

# Stop the service if it is running
if [ -f /etc/init.d/palconnect ]; then
    print_status "Stopping PalConnect service..."
    /etc/init.d/palconnect stop 2>/dev/null || true
fi

# Disable on-boot start
print_status "Disabling PalConnect service..."
if command -v update-rc.d &>/dev/null; then
    update-rc.d -f palconnect remove 2>/dev/null || true
elif command -v chkconfig &>/dev/null; then
    chkconfig palconnect off 2>/dev/null || true
    chkconfig --del palconnect 2>/dev/null || true
fi

# Remove init.d script
if [ -f /etc/init.d/palconnect ]; then
    rm /etc/init.d/palconnect
    print_success "Removed /etc/init.d/palconnect"
fi

# Remove PID file if present
rm -f /var/run/palconnect.pid

print_success "init.d service removed."
echo
print_warning "The following directories were NOT removed (your data is safe):"
echo "  /opt/palconnect   — application binary"
echo "  /etc/palconnect   — configuration files"
echo "  /var/log/palconnect — log files"
echo
print_status "To fully remove all PalConnect files, run:"
echo "  sudo rm -rf /opt/palconnect /etc/palconnect /var/log/palconnect"
echo "  sudo userdel palconnect"
