use std::collections::HashSet;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use log::{error, info, warn};
use rss::Channel;
use tokio::task::JoinHandle;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;

use crate::config::{Config, PalworldServerConfig};
use crate::services::PalworldClient;

const DEFAULT_STEAMDB_RSS_URL: &str = "https://steamdb.info/app/2394010/patchnotes/rss/";

pub fn start_steamdb_updater(
    config: Arc<Config>,
    palworld_client: Arc<PalworldClient>,
    cancellation_token: CancellationToken,
) -> Option<JoinHandle<()>> {
    if !config.steamdb_autoupdate_enabled() {
        info!("⏸️ SteamDB PalWorld autoupdater disabled, skipping background watcher");
        return None;
    }

    if config.multi_tenant() {
        warn!(
            "⏸️ SteamDB PalWorld autoupdater currently only supports single-tenant deployments; skipping watcher"
        );
        return None;
    }

    let servers = config.effective_palworld_servers();
    if servers.is_empty() {
        warn!("⏸️ SteamDB PalWorld autoupdater skipped because no servers are configured");
        return None;
    }

    if config
        .effective_steamcmd_command(&servers[0])
        .is_none()
    {
        warn!("⏸️ SteamDB PalWorld autoupdater skipped because no steamcmd command is configured");
        return None;
    }

    let check_interval = config.update_check_interval().max(60);
    let rss_url = config
        .steamdb_rss_url
        .clone()
        .unwrap_or_else(|| DEFAULT_STEAMDB_RSS_URL.to_string());
    let http = reqwest::Client::new();

    info!(
        "🔄 Starting SteamDB PalWorld autoupdater with {}s interval",
        check_interval
    );

    Some(tokio::spawn(async move {
        let mut seen_updates: HashSet<String> = HashSet::new();
        let mut ticker = interval(Duration::from_secs(check_interval));

        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    info!("🛑 SteamDB autoupdater shutting down gracefully");
                    break;
                }
                _ = ticker.tick() => {
                    match poll_feed(&http, &rss_url).await {
                        Ok(update_ids) => {
                            for update_id in update_ids {
                                if seen_updates.insert(update_id.clone()) {
                                    if let Err(err) = process_update(
                                        &config,
                                        &palworld_client,
                                        &servers[0],
                                        &update_id,
                                    ).await {
                                        error!("❌ Failed to process SteamDB update {}: {}", update_id, err);
                                    }
                                }
                            }
                        }
                        Err(err) => warn!("⚠️ Failed to poll SteamDB RSS feed: {}", err),
                    }
                }
            }
        }
    }))
}

async fn poll_feed(http: &reqwest::Client, rss_url: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let bytes = http.get(rss_url).send().await?.bytes().await?;
    let channel = Channel::read_from(&bytes[..])?;
    Ok(extract_update_ids(&channel))
}

fn extract_update_ids(channel: &Channel) -> Vec<String> {
    channel
        .items()
        .iter()
        .filter_map(|item| {
            item.guid()
                .map(|guid| guid.value().to_string())
                .or_else(|| item.link().map(|link| link.to_string()))
                .or_else(|| item.title().map(|title| title.to_string()))
        })
        .collect()
}

async fn process_update(
    config: &Config,
    palworld_client: &PalworldClient,
    server: &PalworldServerConfig,
    update_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let shutdown_delay = config.update_shutdown_delay();
    let shutdown_message = config.update_shutdown_message();
    let Some(command) = config.effective_steamcmd_command(server) else {
        return Ok(());
    };

    info!("📦 SteamDB reported a PalWorld dedicated server update: {}", update_id);

    if palworld_client.check_online_server(server).await {
        let announce_message = format!(
            "{} Update will begin in {} seconds.",
            shutdown_message, shutdown_delay
        );
        if let Err(err) = palworld_client.announce_server(server, &announce_message).await {
            warn!("⚠️ Failed to announce pending update: {}", err);
        }

        match palworld_client
            .shutdown_server(server, shutdown_delay, shutdown_message)
            .await
        {
            Ok(status) if status.is_success() => {
                info!("🛑 Sent PalWorld shutdown for SteamDB-triggered update");
            }
            Ok(status) => {
                warn!("⚠️ PalWorld shutdown request returned status {}", status);
            }
            Err(err) => {
                warn!("⚠️ Failed to request PalWorld shutdown: {}", err);
            }
        }

        tokio::time::sleep(Duration::from_secs(shutdown_delay)).await;
    }

    info!("⬇️ Running configured steamcmd update command");
    run_command(command)?;
    info!("✅ SteamDB-triggered PalWorld update command completed");
    Ok(())
}

fn run_command(command: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let status = Command::new("sh").arg("-c").arg(command).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("update command exited with status {}", status).into())
    }
}

#[cfg(test)]
mod tests {
    use super::extract_update_ids;

    #[test]
    fn extract_update_ids_prefers_guid_then_link_then_title() {
        let xml = r#"<?xml version="1.0"?>
            <rss version="2.0">
              <channel>
                <title>SteamDB</title>
                <item><guid>guid-1</guid><link>https://example.com/1</link><title>One</title></item>
                <item><link>https://example.com/2</link><title>Two</title></item>
                <item><title>Three</title></item>
              </channel>
            </rss>"#;

        let channel = rss::Channel::read_from(xml.as_bytes()).unwrap();
        assert_eq!(
            extract_update_ids(&channel),
            vec![
                "guid-1".to_string(),
                "https://example.com/2".to_string(),
                "Three".to_string()
            ]
        );
    }
}
