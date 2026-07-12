// 
// PalConnect - A Discord bot for PalWorld server monitoring
// Copyright (C) 2025  Lily Ana Valley <hi@lilyvalley.dev> <https://lilyvalley.dev>
//
// This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General 
// Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) 
// any later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied 
// warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more
// details.
// 
// You should have received a copy of the GNU Affero General Public License along with this program.  If not, see
// <https://www.gnu.org/licenses/>.
// 

use std::fs::File;
use std::io::Read;
use std::env;
use toml;
use serde::Deserialize;
use log::{ trace, debug, error };


const CONFIG_LOCATIONS: [&str; 3] = [
    "./Config.toml",
    "/etc/palconnect/Config.toml",
    "/usr/local/etc/palconnect/Config.toml",
];

// ─── PalWorld server entry ────────────────────────────────────────────────────

/// A single PalWorld dedicated-server definition.
///
/// In the config file these are expressed as `[[palworld_servers]]` table-array entries.
/// The legacy top-level `palworld_api_url` / `palworld_admin_password` fields are converted into a
/// single entry of this type for backward compatibility.
#[derive(Debug, Deserialize, Clone)]
pub struct PalworldServerConfig {
    /// Human-readable label shown in Discord UX (e.g. "Main World").
    pub name: String,
    /// Base URL of the PalWorld REST API, e.g. `http://10.0.0.1:8212`.
    pub api_url: String,
    /// Admin password for HTTP Basic Auth.  Never logged or returned to chat.
    pub admin_password: String,
}

// ─── Top-level Config ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct Config {
    // ── Required ──────────────────────────────────────────────────────────────
    pub discord_token: String,

    // ── Deployment mode ───────────────────────────────────────────────────────
    /// When `true`, the bot operates in multi-tenant mode: tenant and server configuration is
    /// managed through the hosted web app and a database.  When `false` (the default), the bot
    /// runs in single-tenant (self-hosted) mode and reads server credentials from this file.
    pub multi_tenant: Option<bool>,

    /// Whether the bot accepts new guild invites (operator-level gate).
    ///
    /// Defaults to `false` in single-tenant mode (only the first guild join is accepted) and
    /// `true` in multi-tenant mode.  Set explicitly in config to override.
    pub invite_enabled: Option<bool>,

    // ── Multi-server array (preferred) ────────────────────────────────────────
    /// One or more PalWorld server definitions.  Use `[[palworld_servers]]` table-array syntax in
    /// `Config.toml`.  The first entry is treated as the primary server.
    pub palworld_servers: Option<Vec<PalworldServerConfig>>,

    // ── Legacy single-server fields (backward compatible) ─────────────────────
    /// Deprecated in favour of `[[palworld_servers]]`.  Still supported so that existing configs
    /// continue to work without modification.
    pub palworld_api_url: Option<String>,
    /// Deprecated in favour of `[[palworld_servers]]`.  Still supported for backward compatibility.
    pub palworld_admin_password: Option<String>,

    // ── Optional bot settings ─────────────────────────────────────────────────
    pub enable_autoupdate:      Option<bool>,
    pub heartbeat_port:         Option<u16>,
    /// How often (in seconds) the bot polls PalWorld servers and updates status.  Min 15.
    pub status_update_interval: Option<u64>,
    pub logging:                Option<Logging>,
}

impl Config {
    pub fn autoupdate(&self) -> bool {
        self.enable_autoupdate.unwrap_or(false)
    }

    pub fn status_update_interval(&self) -> u64 {
        self.status_update_interval.unwrap_or(30)
    }

    pub fn multi_tenant(&self) -> bool {
        self.multi_tenant.unwrap_or(false)
    }

    pub fn invite_allowed(&self) -> bool {
        self.invite_enabled
            .unwrap_or_else(|| self.multi_tenant()) // default: allowed iff multi-tenant
    }

    /// Return the effective list of PalWorld server definitions.
    ///
    /// Prefers the `[[palworld_servers]]` array; falls back to the legacy `palworld_api_url` /
    /// `palworld_admin_password` fields so that existing configurations continue to work without
    /// any migration.
    pub fn effective_palworld_servers(&self) -> Vec<PalworldServerConfig> {
        if let Some(servers) = &self.palworld_servers {
            if !servers.is_empty() {
                return servers.clone();
            }
        }
        // Legacy fallback
        let url = self.palworld_api_url.clone().unwrap_or_else(|| "http://localhost:8212".to_string());
        let password = self.palworld_admin_password.clone().unwrap_or_default();
        vec![PalworldServerConfig {
            name: "PalWorld Server".to_string(),
            api_url: url,
            admin_password: password,
        }]
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            discord_token:              String::new(),
            multi_tenant:               None,
            invite_enabled:             None,
            palworld_servers:           None,
            palworld_api_url:           Some(String::from("http://localhost:8212/")),
            palworld_admin_password:    Some(String::new()),
            enable_autoupdate:          None,
            heartbeat_port:             None,
            status_update_interval:     None,
            logging:                    None,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct Logging {
    pub use_syslog: Option<bool>,
    pub use_stdout: Option<bool>,
    pub use_file: Option<bool>,
    pub use_file_path: Option<String>,
}

pub fn setup() -> Config {
    
    let mut config_buffer = Vec::new();
    for location in CONFIG_LOCATIONS.iter() {
        trace!("Checking for config file at: {}", location);
        if let Ok(mut file) = File::open(location) {
            trace!("Reading config file at: {}", location);
            file.read_to_end(&mut config_buffer).expect("Failed to read config file");
            trace!("End read.");
            debug!("Using config file at: {}", location);
            break;
        }
    }
    let mut config = toml::de::from_slice(&config_buffer)
        .map(|config: Config| {
            config
        })
        .unwrap_or_else(|e| {
            // * NOTE: If config parsing fails, we return a default config.
            // ? Do we want to parse the remainder of the config locations instead?
            error!("Failed to parse config file: {}", e);
            Config::default()
        });

    // * Load environment variables from .env file
    dotenv::dotenv().ok();

    // * Load environment variables
    if let Ok(discord_token) = env::var("DISCORD_TOKEN") {
        config.discord_token = discord_token;
    }

    // Legacy single-server env vars (still supported for backward compatibility).
    // When PALWORLD_API_URL / PALWORLD_ADMIN_PASSWORD are set they override the legacy fields;
    // if [[palworld_servers]] is also present in the config file, the env vars are ignored.
    if let Ok(palworld_api_url) = env::var("PALWORLD_API_URL") {
        config.palworld_api_url = Some(palworld_api_url);
    }
    
    if let Ok(admin_password) = env::var("PALWORLD_ADMIN_PASSWORD") {
        config.palworld_admin_password = Some(admin_password);
    }

    if let Ok(val) = env::var("MULTI_TENANT") {
        config.multi_tenant = Some(
            val.to_lowercase().parse::<bool>()
                .expect("Failed to parse MULTI_TENANT as bool. Expected 'true' or 'false'"),
        );
    }

    if let Ok(val) = env::var("INVITE_ENABLED") {
        config.invite_enabled = Some(
            val.to_lowercase().parse::<bool>()
                .expect("Failed to parse INVITE_ENABLED as bool. Expected 'true' or 'false'"),
        );
    }
    
    if let Ok(heartbeat_port) = env::var("HEARTBEAT_PORT") {
        config.heartbeat_port = Some(
            heartbeat_port.parse::<u16>()
                .expect("Failed to parse HEARTBEAT_PORT as u16")
        );
    }

    // Check for autoupdate and status interval env vars
    if let Ok(update_enable) = env::var("UPDATES_AUTO_ENABLE") {
        config.enable_autoupdate = Some(
            <bool as std::str::FromStr>::from_str(
                update_enable.to_lowercase().as_str(),
            )
            .expect("Failed to parse UPDATES_AUTO_ENABLE as bool")
        );
    }

    // If the no-autoupdate feature is enabled, disable autoupdate, even if the config or env var says otherwise.
    // This is a preventative measure for prebuilt packages that should not auto-update from within the app itself.
    #[cfg(feature = "no-autoupdate")]
    {
        config.enable_autoupdate = Some(false);
    }

    if let Ok(status_interval) = env::var("STATUS_UPDATE_INTERVAL") {
        let interval = status_interval.parse::<u64>()
            .expect("Failed to parse STATUS_UPDATE_INTERVAL as u64");
        if interval < 15 {
            panic!("STATUS_UPDATE_INTERVAL must be at least 15 seconds to avoid excessive API polling (got {}).", interval);
        }
        config.status_update_interval = Some(interval);
    }

    config

}
