#!/bin/bash

# macOS launchd setup script for PalConnect
# This script sets up a launchd service for PalWorld server

set -e

# Configuration
PALWORLD_USER="palworld"
PALWORLD_DIR="/Applications/PalWorld"
PLIST_NAME="com.palworld.server"
PLIST_PATH="/Library/LaunchDaemons/${PLIST_NAME}.plist"

echo "Setting up PalWorld launchd service on macOS..."

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo "This script must be run as root (use sudo)" 
   exit 1
fi

# Create user if it doesn't exist
if ! id "$PALWORLD_USER" &>/dev/null; then
    echo "Creating user $PALWORLD_USER..."
    # macOS user creation is more complex, this is a basic example
    dscl . -create "/Users/$PALWORLD_USER"
    dscl . -create "/Users/$PALWORLD_USER" UserShell /bin/bash
    dscl . -create "/Users/$PALWORLD_USER" RealName "PalWorld Server"
    dscl . -create "/Users/$PALWORLD_USER" UniqueID 502
    dscl . -create "/Users/$PALWORLD_USER" PrimaryGroupID 20
    dscl . -create "/Users/$PALWORLD_USER" NFSHomeDirectory /var/empty
fi

# Copy the plist file (assumes it exists in examples directory)
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
if [[ -f "$SCRIPT_DIR/com.palworld.server.plist" ]]; then
    echo "Installing launchd plist..."
    cp "$SCRIPT_DIR/com.palworld.server.plist" "$PLIST_PATH"
    chown root:wheel "$PLIST_PATH"
    chmod 644 "$PLIST_PATH"
else
    echo "Warning: com.palworld.server.plist not found in examples directory"
    echo "Please manually copy the plist file to $PLIST_PATH"
fi

# Create log directory
mkdir -p /var/log
touch /var/log/palworld.log /var/log/palworld_error.log
chown "$PALWORLD_USER:staff" /var/log/palworld*.log

echo "Launchd service has been installed."
echo "Load it with: sudo launchctl load -w $PLIST_PATH"
echo "Unload it with: sudo launchctl unload -w $PLIST_PATH"
echo ""
echo "Configure PalConnect with:"
echo "[server_management]"
echo "service_type = \"launchd\""
echo "service_name = \"$PLIST_PATH\""
