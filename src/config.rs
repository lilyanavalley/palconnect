
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
    pub logging:                    Option<Logging>,
}

impl Config {
    pub fn autoupdate(&self) -> bool {
        self.enable_autoupdate.unwrap_or(false)
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

    config

}
