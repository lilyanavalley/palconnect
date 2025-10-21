#!/bin/bash
# Quick test script to verify PalWorld REST API authentication

echo "Testing PalWorld REST API authentication..."
echo "Make sure your PalWorld server is running with REST API enabled"
echo ""

# Default values - update these as needed
PALWORLD_URL="http://localhost:8212"
ADMIN_PASSWORD="test"  # Your AdminPassword from server config

echo "Testing /v1/api/info endpoint..."
curl -u "admin:$ADMIN_PASSWORD" \
     -H "Accept: application/json" \
     "$PALWORLD_URL/v1/api/info" \
     -w "\nStatus Code: %{http_code}\n" \
     2>/dev/null

echo ""
echo "Testing /v1/api/players endpoint..."
curl -u "admin:$ADMIN_PASSWORD" \
     -H "Accept: application/json" \
     "$PALWORLD_URL/v1/api/players" \
     -w "\nStatus Code: %{http_code}\n" \
     2>/dev/null

echo ""
echo "If you see JSON responses above, authentication is working!"
echo "If you get 401 errors, check your AdminPassword in the server config."
