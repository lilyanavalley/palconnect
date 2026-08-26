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
# PalConnect Systemd Service Test Script
# This script tests the systemd service installation and basic functionality
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

print_test() {
    echo -e "${YELLOW}[TEST]${NC} $1"
}

echo "🔍 PalConnect Systemd Service Test"
echo "=================================="
echo

# Check if service exists
print_test "Checking if systemd service exists..."
if systemctl list-unit-files | grep -q palconnect.service; then
    print_success "Systemd service file found"
else
    print_error "Systemd service file not found"
    exit 1
fi

# Check service status
print_test "Checking service status..."
if systemctl is-active --quiet palconnect.service; then
    print_success "Service is active"
    SERVICE_ACTIVE=true
else
    print_warning "Service is not active"
    SERVICE_ACTIVE=false
fi

# Check if service is enabled
print_test "Checking if service is enabled..."
if systemctl is-enabled --quiet palconnect.service; then
    print_success "Service is enabled for auto-start"
else
    print_warning "Service is not enabled for auto-start"
fi

# Check binary exists
print_test "Checking binary installation..."
if [ -f /opt/palconnect/palconnect ]; then
    print_success "Binary found at /opt/palconnect/palconnect"
else
    print_error "Binary not found at /opt/palconnect/palconnect"
    exit 1
fi

# Check configuration file
print_test "Checking configuration file..."
if [ -f /etc/palconnect/palconnect.env ]; then
    print_success "Configuration file found"
    
    # Check if Discord token is configured
    if grep -q "your_discord_bot_token_here" /etc/palconnect/palconnect.env 2>/dev/null; then
        print_warning "Discord token appears to be unconfigured (still contains placeholder)"
    else
        print_success "Discord token appears to be configured"
    fi
else
    print_error "Configuration file not found at /etc/palconnect/palconnect.env"
fi

# Check user exists
print_test "Checking palconnect user..."
if id palconnect >/dev/null 2>&1; then
    print_success "palconnect user exists"
else
    print_error "palconnect user does not exist"
fi

# Check permissions
print_test "Checking file permissions..."
if [ -O /opt/palconnect/palconnect ] && [ -G /opt/palconnect/palconnect ]; then
    print_success "Binary has correct ownership"
else
    print_warning "Binary ownership may be incorrect"
fi

# Test health endpoint (if service is running)
if [ "$SERVICE_ACTIVE" = true ]; then
    print_test "Testing health endpoint..."
    if curl -s -f http://localhost:3000/health >/dev/null 2>&1; then
        print_success "Health endpoint is responding"
    else
        print_warning "Health endpoint is not responding (service may still be starting)"
    fi
else
    print_test "Attempting to start service for testing..."
    if systemctl start palconnect.service 2>/dev/null; then
        print_success "Service started successfully"
        sleep 5
        
        print_test "Testing health endpoint after start..."
        if curl -s -f http://localhost:3000/health >/dev/null 2>&1; then
            print_success "Health endpoint is responding"
        else
            print_warning "Health endpoint is not responding"
        fi
    else
        print_error "Failed to start service for testing"
    fi
fi

# Check recent logs for errors
print_test "Checking recent logs for errors..."
ERROR_COUNT=$(journalctl -u palconnect --since "10 minutes ago" -p err --no-pager -q | wc -l)
if [ "$ERROR_COUNT" -eq 0 ]; then
    print_success "No recent errors found in logs"
else
    print_warning "Found $ERROR_COUNT error(s) in recent logs"
    echo "Recent errors:"
    journalctl -u palconnect --since "10 minutes ago" -p err --no-pager | head -5
fi

echo
echo "🏁 Test Summary"
echo "==============="

# Show service status
systemctl status palconnect.service --no-pager --lines=5 2>/dev/null || echo "Could not retrieve service status"

echo
print_status "For more detailed logs, run: sudo journalctl -u palconnect -f"
print_status "To check service status: sudo systemctl status palconnect"
