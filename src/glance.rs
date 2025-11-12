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

use log::{debug, error, info, warn};
use poise::serenity_prelude as serenity;
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

use crate::{BotData, Error};

#[derive(Debug, Deserialize)]
struct PlayersResponse {
    players: Vec<Player>,
}

#[derive(Debug, Deserialize)]
struct Player {
    name: String,
    #[serde(rename = "playerId")]
    player_id: String,
    #[serde(rename = "userId")]
    user_id: String,
    ip: String,
    ping: f64,
    location_x: f64,
    location_y: f64,
    level: u32,
}

#[derive(Debug, Deserialize)]
struct ServerInfo {
    version: String,
    servername: String,
    description: String,
}

/// Start the background task that updates the bot's status with server information
pub async fn start_status_updater(
    ctx: Arc<serenity::Context>,
    bot_data: Arc<BotData>,
    update_interval_seconds: u64,
) {
    info!("🔄 Starting status updater with {}s interval", update_interval_seconds);
    
    let mut interval_timer = interval(Duration::from_secs(update_interval_seconds));
    
    // Set initial status
    ctx.set_activity(Some(serenity::ActivityData::playing("/help - PalConnect Commands")));
    
    tokio::spawn(async move {
        loop {
            interval_timer.tick().await;
            
            match update_bot_status(&ctx, &bot_data).await {
                Ok(_) => debug!("✅ Status updated successfully"),
                Err(e) => error!("❌ Failed to update status: {}", e),
            }
        }
    });
}

/// Update the bot's status with current server information
async fn update_bot_status(
    ctx: &Arc<serenity::Context>,
    bot_data: &Arc<BotData>,
) -> Result<(), Error> {
    // First try to get player count
    match get_player_count(&bot_data.http_client, &bot_data.palworld_api_url, &bot_data.admin_password).await {
        Ok(player_count) => {
            // Try to get server info for max players (if available)
            match get_server_info(&bot_data.http_client, &bot_data.palworld_api_url, &bot_data.admin_password).await {
                Ok(server_info) => {
                    // Show player count with server name
                    let status = format!("{} players on {} | /help", player_count, server_info.servername);
                    ctx.set_activity(Some(serenity::ActivityData::watching(status)));
                    debug!("Status updated: {} players on {}", player_count, server_info.servername);
                }
                Err(_) => {
                    // Fallback to just player count
                    let status = format!("{} players online | /help", player_count);
                    ctx.set_activity(Some(serenity::ActivityData::watching(status)));
                    debug!("Status updated: {} players (server info unavailable)", player_count);
                }
            }
        }
        Err(e) => {
            // Server unreachable, show offline status
            warn!("Server unreachable: {}", e);
            ctx.set_activity(Some(serenity::ActivityData::playing("Server offline | /help")));
        }
    }
    
    Ok(())
}

/// Get current player count from the PalWorld server
async fn get_player_count(
    http_client: &Client,
    palworld_api_url: &str,
    admin_password: &str,
) -> Result<usize, Error> {
    let url = format!("{}/v1/api/players", palworld_api_url);
    
    let response = http_client
        .get(&url)
        .basic_auth("admin", Some(admin_password))
        .timeout(Duration::from_secs(10)) // 10 second timeout
        .send()
        .await?;
    
    let players_data: PlayersResponse = response.json().await?;
    Ok(players_data.players.len())
}

/// Get server information from the PalWorld server
async fn get_server_info(
    http_client: &Client,
    palworld_api_url: &str,
    admin_password: &str,
) -> Result<ServerInfo, Error> {
    let url = format!("{}/v1/api/info", palworld_api_url);
    
    let response = http_client
        .get(&url)
        .basic_auth("admin", Some(admin_password))
        .timeout(Duration::from_secs(10)) // 10 second timeout
        .send()
        .await?;
    
    let server_info: ServerInfo = response.json().await?;
    Ok(server_info)
}

/// Manually trigger a status update (useful for testing or immediate updates)
pub async fn update_status_now(
    ctx: &Arc<serenity::Context>,
    bot_data: &Arc<BotData>,
) -> Result<(), Error> {
    update_bot_status(ctx, bot_data).await
}
