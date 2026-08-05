//
//  ,ggggggggggg,                   ,gggg,
// dP"""88""""""Y8,      ,dPYb,   ,88"""Y8b,                                                            I8
// Yb,  88      `8b      IP'`Yb  d8"     `Y8                                                            I8
//  `"  88      ,8P      I8  8I d8'   8b  d8                                                         88888888
//      88aaaad8P"       I8 dP I8'            ,ggggg,    ,ggg,,ggg,    ,ggg,,ggg,    ,ggg,     ,gggg,   I8
//      88""""",gggg,gg  I8 dP  d8            dP"  "Y8ggg,8" "8P" "8,  ,8" "8P" "8,  i8" "8i   dP"  "Yb  I8
//      88    dP"  "Y8I  I8P   Y8,          i8'    ,8I  I8   8I   8I  I8   8I   8I  I8, ,8I  i8'       ,I8,
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
use clap::Parser;
use fern;
#[cfg(unix)]
use fork;
use log::{error, info, warn};
use std::fs;
use std::io::Write;

use palconnect_bot::{Error, dispatcher};

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
