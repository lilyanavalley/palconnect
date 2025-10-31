
# PalConnect

A Discord bot that connects to PalWorld Dedicated Server REST API to show real-time player information and server status.

> [!WARNING]
> PalConnect is a work in progress!

## Features

- 🎮 **Real-time Player Count**: See how many players are currently online
- 👥 **Player List**: View all online players with their levels
- 🏰 **Server Information**: Display server name, version, and description
- 🔄 **Live Updates**: Real-time data from your PalWorld server

## Commands

- `/players` - Show current online players and count
- `/serverinfo` - Display server information
- `/help` - Show help message

## Setup

### 1. Prerequisites

- Rust (latest stable version)
- A Discord application/bot token
- A PalWorld Dedicated Server with REST API enabled

### 2. Discord Bot Setup

1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. Create a new application
3. Go to the "Bot" section
4. Create a bot and copy the token
5. Under "OAuth2" → "URL Generator":
   - Select `bot` and `applications.commands` scopes
   - Select appropriate bot permissions (Send Messages, Use Slash Commands, etc.)
   - Use the generated URL to invite the bot to your server

### 3. PalWorld Server Configuration

Make sure your PalWorld dedicated server has the REST API enabled and has an admin password set. In your server configuration:

```ini
RESTAPIEnabled=True
RESTAPIPort=8212
AdminPassword=your_secure_admin_password_here
```

**Important**: The REST API uses HTTP Basic Authentication where:
- Username: `admin` (fixed)
- Password: Your `AdminPassword` value from the server config

### 4. Environment Setup

1. Copy the example environment file:
   ```bash
   cp .env.example .env
   ```

2. Edit `.env` and add your configuration:
   ```env
   DISCORD_TOKEN=your_discord_bot_token_here
   PALWORLD_API_URL=http://your-server-ip:8212
   PALWORLD_ADMIN_PASSWORD=your_admin_password_here
   ```
   
   **Note**: Use the same password you set as `AdminPassword` in your PalWorld server config.

### 5. Running the Bot

1. Install dependencies and run:
   ```bash
   cargo run
   ```

The bot will automatically register slash commands when it starts up.

## Development

### Project Structure

- `src/main.rs` - Main bot code with commands and API integration
- `Cargo.toml` - Rust dependencies and project configuration
- `.env` - Environment variables (create from `.env.example`)

### Adding New Commands

Commands are defined using the `#[poise::command(slash_command)]` attribute. Add new commands to the `commands` vector in the framework builder.

### PalWorld API Endpoints

The bot currently uses these PalWorld REST API endpoints:
- `GET /v1/api/players` - Get current players
- `GET /v1/api/info` - Get server information

## Releases & Distribution

PalConnect uses automated GitHub Actions for building and releasing:

### Installation

Download the latest release for your platform:

- **Windows**: `.msi` installer
- **macOS**: `.dmg` package  
- **Linux**: `.deb` package

All releases are cryptographically signed for security.

### Linux Server Deployment

For Linux servers, PalConnect can be deployed as a systemd service:

```bash
# Quick installation
sudo ./systemd/install.sh

# Configure environment
sudo nano /etc/palconnect/palconnect.env

# Start service
sudo systemctl start palconnect
```

See [systemd/README.md](systemd/README.md) for complete systemd deployment documentation.

### Auto-Updates

The application includes automatic update functionality via `cargo-packager-updater`:

- Checks for updates on startup
- Verifies cryptographic signatures before updating
- Prompts user before installing updates

### For Developers

See [docs/RELEASE.md](docs/RELEASE.md) for complete release process documentation, including:

- Setting up package signing
- Creating releases
- Understanding the build pipeline

See [docs/SIGNING.md](docs/SIGNING.md) for documentation on release signing.

## Troubleshooting

### Common Issues

1. **"Failed to connect to PalWorld server"**
   - Check that your PalWorld server is running
   - Verify the REST API is enabled and the port is correct
   - Ensure the `PALWORLD_API_URL` is correct

2. **"Expected DISCORD_TOKEN environment variable"**
   - Make sure you've created a `.env` file with your Discord bot token
   - Verify the token is correct and the bot has proper permissions

3. **Slash commands not appearing**
   - Wait a few minutes for Discord to register the commands globally
   - Check that the bot has the `applications.commands` scope

## License

This project is open source under [AGPL-3](https://www.gnu.org/licenses/agpl-3.0.html#license-text).

## Contributing

Feel free to submit issues and pull requests!
