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
# Quick test script to verify PalWorld REST API authentication
#######################################################################################################################

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
