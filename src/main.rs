
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use cargo_packager_updater;
use clap::Parser;
use fern;
#[cfg(unix)]
use fork;
use log::{debug, error, info, warn};
use poise::serenity_prelude as serenity;
use reqwest::Client;
use serde::Deserialize;
use std::fs;
use std::io::Write;
#[cfg(unix)]
use syslog;
use tokio::signal;

mod config;
use config::*;
mod commands;
use commands::*;
mod health_check;
use health_check::*;


type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, BotData, Error>;

const UPDATE_ENDPOINT: &str =
    "https://github.com/lilyanavalley/palconnect/releases/download/updates";
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
pub struct BotData {
    http_client: Client,
    palworld_api_url: String,
    admin_password: String,
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
            return Ok(());
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
    if config.autoupdate() {
        info!("🔄 Autoupdate enabled, checking online for newer copy...");
        info!("Current version number: {}", env!("CARGO_PKG_VERSION"));

        let config = cargo_packager_updater::Config {
            endpoints: vec![UPDATE_ENDPOINT.parse().unwrap()],
            pubkey: UPDATE_PUBKEY.into(),
            ..Default::default()
        };

        match cargo_packager_updater::check_update(
            env!("CARGO_PKG_VERSION").parse().unwrap(),
            config,
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
    }

    if !config.autoupdate() {
        info!("⏸️ Autoupdate disabled, skipping update check");
    }

    info!("🚀 Starting PalConnect bot...");
    info!("📡 PalWorld API URL: {}", config.palworld_api_url);

    // * Setup Discord bot
    let framework_poise = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![players(), serverinfo(), help()],
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            let palworld_api_url = config.palworld_api_url.clone();
            let admin_password = config.palworld_admin_password.clone();
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(BotData {
                    http_client: Client::new(),
                    palworld_api_url,
                    admin_password,
                })
            })
        })
        .build();

    let poise_intents = serenity::GatewayIntents::non_privileged();
    let mut poise_client = serenity::ClientBuilder::new(config.discord_token, poise_intents)
        .framework(framework_poise)
        .await
        .expect("Failed to create Discord client");

    // * Setup Actix Web server
    let actix_server = HttpServer::new(|| App::new().service(health_check))
        .bind(("0.0.0.0", config.heartbeat_port.unwrap_or(8080)))?
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
