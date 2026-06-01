// 
//  ,ggggggggggg,                   ,gggg,                                                                   
// dP"""88""""""Y8,      ,dPYb,   ,88"""Y8b,                                                            I8   
// Yb,  88      `8b      IP'`Yb  d8"     `Y8                                                            I8   
//  `"  88      ,8P      I8  8I d8'   8b  d8                                                         88888888
//      88aaaad8P"       I8  8',8I    "Y88P'                                                            I8   
//      88""""",gggg,gg  I8 dP I8'            ,ggggg,    ,ggg,,ggg,    ,ggg,,ggg,    ,ggg,     ,gggg,   I8   
//      88    dP"  "Y8I  I8dP  d8            dP"  "Y8ggg,8" "8P" "8,  ,8" "8P" "8,  i8" "8i   dP"  "Yb  I8   
//      88   i8'    ,8I  I8P   Y8,          i8'    ,8I  I8   8I   8I  I8   8I   8I  I8, ,8I  i8'       ,I8,  
//      88  ,d8,   ,d8b,,d8b,_ `Yba,,_____,,d8,   ,d8' ,dP   8I   Yb,,dP   8I   Yb, `YbadP' ,d8,_    _,d88b, 
//      88  P"Y8888P"`Y88P'"Y88  `"Y8888888P"Y8888P"   8P'   8I   `Y88P'   8I   `Y8888P"Y888P""Y8888PP8P""Y8 
// 
//                                A Discord bot for PalWorld server monitoring
// 
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// 
// © Lily Ana Valley <hi@lilyvalley.dev>, 2025
// 🪪 LICENSE: AGPL-3
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
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// 
/// PalConnect is a cross-server connector between a PalWorld dedicated server and a Discord server.
/// Using the REST API of a PalWorld server, it's possible to administrate one's world and allow players to check on 
/// their world from Discord.
/// 
/// 🚧 PalConnect is pre-release software until version `1.0.0` is published. Observations of bot instability, feature 
/// changes and inconsistency is to be expected.
/// 
/// Leave your feedback on the [GitHub Repo](https://github.com/lilyanavalley/palconnect) to help improve this
/// software.
///
// TODO: include documentation on *how* to use this app.

use actix_web::{App, HttpServer};
use cargo_packager_updater;
use clap::Parser;
use fern;
#[cfg(unix)]
use fork;
use log::{debug, error, info, warn};
use poise::serenity_prelude as serenity;
use reqwest::Client;
use std::fs;
use std::io::Write;
use std::sync::{Arc, Mutex};
#[cfg(unix)]
use syslog;
use tokio::signal;
use tokio_util::sync::CancellationToken;

mod config;
use config::*;
mod glance;
use glance::*;
mod commands;
use commands::*;
mod health_check;
use health_check::*;
mod service;
pub use service::ServiceManager;


type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, BotData, Error>;

const UPDATE_ENDPOINT: &str =
    "https://raw.githubusercontent.com/lilyanavalley/palconnect/refs/heads/live/.updater/latest.json";
const UPDATE_PUBKEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDNDOTAzRTg4OUIwN0QwMzEKUldReDBBZWJpRDZRUE40MVFVUklML3g4aVFFRTgvSTlad3hjWDl5UUljbFNEVGJUei9uL0M1SFEK";


#[derive(Parser)]
#[command(name = "palconnect")]
#[command(about = "A Discord bot for PalWorld server monitoring")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Args {
    /// Run as a daemon in the background (Unix only)
    #[cfg(unix)]
    #[arg(short, long)]
    daemon: bool,
}

// Data structure that will be accessible in all command invocations
#[derive(Clone)]
pub struct BotData {
    http_client: Client,
    palworld_api_url: String,
    admin_password: String,
    pub palworld_service_name: String,
    pub palworld_service_manager: String,
}

/// Handles the daemonization process on Unix platforms.
/// Forks the process into a background daemon, writes a PID file, runs the dispatcher,
/// and cleans up the PID file on exit.
#[cfg(unix)]
async fn handle_daemon_mode() -> Result<(), Error> {
    info!("👹 Starting in daemon mode...");
    match fork::daemon(false, false) {
        Ok(fork::Fork::Child) => {
            // We are in the child process (daemon)
            let pid = std::process::id();
            info!("🔧 Daemon process started with PID: {}", pid);

            // Write PID file
            if let Err(e) = write_pid_file(pid) {
                warn!("⚠️ Failed to write PID file: {}", e);
            }

            let result = dispatcher().await;

            // Clean up PID file on exit
            if let Err(e) = remove_pid_file() {
                warn!("⚠️ Failed to remove PID file: {}", e);
            }

            result.expect("Failed to run dispatcher in daemon mode");
        }
        Ok(fork::Fork::Parent(_child_pid)) => {
            // We are in the parent process - exit cleanly
            info!("🚀 Daemon started successfully");
        }
        Err(e) => {
            error!("❌ Failed to daemonize: {}", e);
            return Err(format!("Failed to daemonize: {}", e).into());
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // * Initialize logging first thing (stdout and file on all platforms)
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}]: {}",
                record.level(),
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file("log.txt")?)
        .apply()
        .expect("Failed to initialize logging");

    info!("🚀 PalConnect starting up...");
    let args = Args::parse();

    #[cfg(unix)]
    {
        info!("🐧 Unix platform detected");
        if args.daemon {
            return handle_daemon_mode().await;
        } else {
            info!("🖥️ Running in foreground mode");
        }
    }

    dispatcher().await
}

async fn dispatcher() -> Result<(), Error> {
    info!("🔧 Starting main application dispatcher...");
    
    let config = setup();

    // * Check for updates and apply if available
    check_and_install_updates(&config).await;

    info!("🚀 Starting PalConnect bot...");
    info!("📡 PalWorld API URL: {}", config.palworld_api_url);

    // Store config values we need later before moving config
    let heartbeat_port = config.heartbeat_port.unwrap_or(8080);
    let discord_token = config.discord_token.clone();

    start_services(config, discord_token, heartbeat_port).await
}

/// Checks for available updates and installs them if autoupdate is enabled.
/// Uses cargo-packager-updater to check for new versions and install them.
async fn check_and_install_updates(config: &Config) {
    if config.autoupdate() {
        info!("🔄 Autoupdate enabled, checking online for newer copy...");
        info!("Current version number: {}", env!("CARGO_PKG_VERSION"));

        let updater_config = cargo_packager_updater::Config {
            endpoints: vec![UPDATE_ENDPOINT.parse().unwrap()],
            pubkey: UPDATE_PUBKEY.into(),
            ..Default::default()
        };

        match cargo_packager_updater::check_update(
            env!("CARGO_PKG_VERSION").parse().unwrap(),
            updater_config,
        ) {
            Ok(Some(update)) => {
                info!("⬇️ Update found, downloading and installing...");
                debug!("New version number: {}", update.version);
                debug!("New version signature: {}", update.signature);
                debug!("New version publish date: {:?}", update.date);
                debug!("New version target: {}", update.target);

                match update.download_and_install() {
                    Ok(_) => {
                        info!("🔄 Update installed successfully, restarting...");
                        // This should restart the application
                    }
                    Err(e) => {
                        error!(
                            "🔺 Update installation failed: {}, continuing with current version",
                            e
                        );
                    }
                }
            }
            Ok(None) => {
                info!("✅ No updates found, continuing startup...");
            }
            Err(e) => {
                error!("🔺 Failed to check for updates: {}, continuing startup", e);
            }
        }
    } else {
        info!("⏸️ Autoupdate disabled, skipping update check");
    }
}

/// Starts the Discord bot and Actix web server concurrently.
/// Sets up the Discord bot with commands, starts a status updater background task,
/// and runs the health check server. Handles graceful shutdown on Ctrl+C.
async fn start_services(
    config: Config,
    discord_token: String,
    heartbeat_port: u16,
) -> Result<(), Error> {
    // Create cancellation token and JoinHandle storage for graceful shutdown
    let cancellation_token = CancellationToken::new();
    let status_updater_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>> = Arc::new(Mutex::new(None));
    let status_updater_handle_clone = status_updater_handle.clone();
    let cancellation_token_clone = cancellation_token.clone();

    // * Setup Discord bot
    let framework_poise = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                about(),
                players(),
                serverinfo(),
                help(),
                start(),
                stop(),
                forcestop(),
                settings(),
                metrics(),
                announce(),
                kick(),
                ban(),
                unban(),
                save(),
                update_status(),
            ],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            let palworld_api_url = config.palworld_api_url.clone();
            let admin_password = config.palworld_admin_password.clone();
            let status_interval = config.status_update_interval();
            let palworld_service_name = config.palworld_service_name().to_string();
            let palworld_service_manager = config.palworld_service_manager().to_string();
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                
                let bot_data = BotData {
                    http_client: Client::new(),
                    palworld_api_url,
                    admin_password,
                    palworld_service_name,
                    palworld_service_manager,
                };
                
                // Start the status updater background task with Arc-wrapped data
                let ctx_arc = std::sync::Arc::new(ctx.clone());
                let bot_data_arc = std::sync::Arc::new(bot_data.clone());
                let handle = start_status_updater(ctx_arc, bot_data_arc, status_interval, cancellation_token_clone).await;
                
                // Store the JoinHandle for graceful shutdown
                *status_updater_handle_clone.lock().unwrap() = Some(handle);
                
                Ok(bot_data)
            })
        })
        .build();

    let poise_intents = serenity::GatewayIntents::non_privileged();
    let mut poise_client = serenity::ClientBuilder::new(discord_token, poise_intents)
        .framework(framework_poise)
        .await
        .expect("Failed to create Discord client");

    // * Setup Actix Web server
    let actix_server = HttpServer::new(|| App::new().service(health_check))
        .bind(("0.0.0.0", heartbeat_port))?
        .run();

    info!("✅ Starting both Discord bot and health check server...");

    // * Run both services concurrently with graceful shutdown
    tokio::select! {
        result = poise_client.start() => {
            error!("Discord bot stopped: {:?}", result);
            result?;
        }
        result = actix_server => {
            error!("Actix server stopped: {:?}", result);
            result?;
        }
        _ = signal::ctrl_c() => {
            info!("🛑 Received Ctrl+C, shutting down gracefully...");
        }
    }

    // Cancel the status updater and wait for it to finish
    info!("🛑 Cancelling status updater...");
    cancellation_token.cancel();
    
    let handle = status_updater_handle.lock().unwrap().take();
    if let Some(handle) = handle {
        match tokio::time::timeout(std::time::Duration::from_secs(5), handle).await {
            Ok(_) => info!("✅ Status updater stopped gracefully"),
            Err(_) => warn!("⚠️ Status updater did not stop within timeout"),
        }
    }

    info!("👋 Shutdown complete");
    Ok(())
}

#[cfg(unix)]
fn write_pid_file(pid: u32) -> std::io::Result<()> {
    let pid_path = "/tmp/palconnect.pid";
    let mut file = fs::File::create(pid_path)?;
    writeln!(file, "{}", pid)?;
    info!("📄 PID file written to {}", pid_path);
    Ok(())
}

#[cfg(unix)]
fn remove_pid_file() -> std::io::Result<()> {
    let pid_path = "/tmp/palconnect.pid";
    if std::path::Path::new(pid_path).exists() {
        fs::remove_file(pid_path)?;
        info!("🗑️ PID file removed");
    }
    Ok(())
}
