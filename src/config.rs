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
use log::{ trace, debug, info, warn, error };


const CONFIG_LOCATIONS: [&str; 3] = [
    "./Config.toml",
    "/etc/palconnect/Config.toml",
    "/usr/local/etc/palconnect/Config.toml",
];


#[derive(Debug, Deserialize)]
pub struct Config {
    pub discord_token:              String,             // * Required
    pub palworld_api_url:           String,             // * Required
    pub palworld_admin_password:    String,             // * Required
    pub enable_autoupdate:          Option<bool>,
    pub heartbeat_port:             Option<u16>,
    pub status_update_interval:     Option<u64>,        // * Status update interval in seconds
    pub logging:                    Option<Logging>,
    pub server_management:          Option<ServerManagement>,
}

impl Config {
    pub fn autoupdate(&self) -> bool {
        self.enable_autoupdate.unwrap_or(false)
    }
    
    pub fn status_update_interval(&self) -> u64 {
        self.status_update_interval.unwrap_or(30) // Default to 30 seconds
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            discord_token:              String::new(),
            palworld_api_url:           String::from("http://localhost:8212/"),
            palworld_admin_password:    String::new(),
            enable_autoupdate:          None,
            heartbeat_port:             None,
            status_update_interval:     None,
            logging:                    None,
            server_management:          None,
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

#[derive(Debug, Clone, Deserialize)]
pub struct ServerManagement {
    pub service_type: ServiceType,
    pub service_name: Option<String>,
    pub start_command: Option<String>,
    pub stop_command: Option<String>,
    pub force_stop_command: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceType {
    Systemd,
    Launchd,
    WindowsService,
    CustomScript,
    PowerShell,
}

impl Default for ServerManagement {
    fn default() -> Self {
        #[cfg(target_os = "linux")]
        let default_service_type = ServiceType::Systemd;
        
        #[cfg(target_os = "macos")]
        let default_service_type = ServiceType::Launchd;
        
        #[cfg(target_os = "windows")]
        let default_service_type = ServiceType::WindowsService;
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        let default_service_type = ServiceType::CustomScript;

        ServerManagement {
            service_type: default_service_type,
            service_name: None,
            start_command: None,
            stop_command: None,
            force_stop_command: None,
        }
    }
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

    if let Ok(palworld_api_url) = env::var("PALWORLD_API_URL") {
        config.palworld_api_url = palworld_api_url;
    }
    
    if let Ok(admin_password) = env::var("PALWORLD_ADMIN_PASSWORD") {
        config.palworld_admin_password = admin_password;
    }
    
    if let Ok(heartbeat_port) = env::var("HEARTBEAT_PORT") {
        config.heartbeat_port = Some(
            heartbeat_port.parse::<u16>()
                .expect("Failed to parse HEARTBEAT_PORT as u16")
        );
    }

    if let Ok(update_enable) = env::var("UPDATES_AUTO_ENABLE") {
        config.enable_autoupdate = Some(
            <bool as std::str::FromStr>::from_str(
                update_enable.to_lowercase().as_str(),
            )
            .expect("Failed to parse UPDATES_AUTO_ENABLE as bool")
        );
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
