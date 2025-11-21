#!/bin/bash

# Example Linux systemd setup script for PalConnect
# This script creates a systemd service for PalWorld server

set -e

# Configuration
PALWORLD_USER="palworld"
PALWORLD_HOME="/home/palworld"
SERVER_DIR="$PALWORLD_HOME/server"
SERVICE_NAME="palworld.service"

echo "Setting up PalWorld systemd service..."

# Create user if it doesn't exist
if ! id "$PALWORLD_USER" &>/dev/null; then
    echo "Creating user $PALWORLD_USER..."
    sudo useradd -r -m -d "$PALWORLD_HOME" -s /bin/bash "$PALWORLD_USER"
fi

# Create service file
echo "Creating systemd service file..."
sudo tee "/etc/systemd/system/$SERVICE_NAME" > /dev/null <<EOF
[Unit]
Description=PalWorld Dedicated Server
After=network.target

[Service]
Type=simple
User=$PALWORLD_USER
Group=$PALWORLD_USER
WorkingDirectory=$SERVER_DIR
ExecStart=$SERVER_DIR/PalServer -port=8211 -players=32
Restart=always
RestartSec=5
Environment=XDG_RUNTIME_DIR=/run/user/1001

[Install]
WantedBy=multi-user.target
EOF

# Set permissions
sudo chown "$PALWORLD_USER:$PALWORLD_USER" "$PALWORLD_HOME" -R 2>/dev/null || true

# Enable and start the service
echo "Enabling systemd service..."
sudo systemctl daemon-reload
sudo systemctl enable "$SERVICE_NAME"

echo "Service $SERVICE_NAME has been created and enabled."
echo "You can now start it with: sudo systemctl start $SERVICE_NAME"
echo ""
echo "Configure PalConnect with:"
echo "[server_management]"
echo "service_type = \"systemd\""
echo "service_name = \"$SERVICE_NAME\""
