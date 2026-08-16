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

use poise::serenity_prelude as serenity;
use serde::Deserialize;
use tracing::{ instrument, info, warn, error, debug, trace, trace_span };

use crate::{Context, Error};


#[derive(Debug, Deserialize)]
struct PlayersResponse {
    players: Vec<Player>,
}

#[derive(Debug, Deserialize)]
struct Player {

    name: String,

    // * Added post-v1 Palworld update
    #[serde(rename = "accountName")]
    account_name: String,
    
    #[serde(rename = "playerId")]
    player_id: String,
    
    #[serde(rename = "userId")]
    user_id: String,
    
    ip: String, // * Currently unused
    
    ping: f64,
    
    location_x: f64,
    
    location_y: f64,
    
    level: u32,

    // * Added post-v1 Palworld update
    building_count: u32,

}

/// Show current player count on the PalWorld server
#[instrument(skip(ctx))]
#[poise::command(slash_command)]
pub async fn players(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let data = ctx.data();
    let url = format!("{}/v1/api/players", data.palworld_api_url);

    match data
        .http_client
        .get(&url)
        .basic_auth("admin", Some(&data.admin_password))
        .send()
        .await
    {
        Ok(response) => match response.json::<PlayersResponse>().await {
            Ok(players_data) => {
                debug!("Received /players response, vector length: {}", players_data.players.len());
                let player_count = players_data.players.len();
                let player_list = if players_data.players.is_empty() {
                    "No players currently online".to_string()
                } else {
                    trace_span!("Formatting player list").in_scope(|| {
                        debug!("Formatting player list for {} players", player_count);
                        players_data
                            .players
                            .iter()
                            .map(|p| format!(
                                "{name} \n |---- 🌐 Account: {account} | 🆔 Player: {player_id} \n |---- 🯊 Level: {level} | 🏛 Buildings: {building_count} \n |---- 📍 Location: ({location_x}, {location_y}) | 📶 Ping: {ping}ms \n",
                                name = p.name,
                                account = p.account_name,
                                player_id = p.player_id,
                                level = p.level,
                                building_count = p.building_count,
                                location_x = p.location_x,
                                location_y = p.location_y,
                                ping = p.ping
                            ))
                            .collect::<Vec<String>>()
                            .join("\n")
                    })
                };

                let embed = serenity::CreateEmbed::new()
                    .title("🎮 PalWorld Server Status")
                    .field("Players Online", player_count.to_string(), true)
                    .field("Player List", player_list, false)
                    .color(if player_count > 0 { 0x00ff00 } else { 0xff0000 }) // Green if players online, red if none
                    .timestamp(serenity::Timestamp::now());

                ctx.send(poise::CreateReply::default().embed(embed)).await?;
            }
            Err(e) => {
                error!("Failed to parse server response: {}", e);
                ctx.send(
                    poise::CreateReply::default()
                        .content(format!("❌ Failed to parse server response: {}", e))
                        .ephemeral(true),
                )
                .await?;
            }
        },
        Err(e) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ Failed to connect to PalWorld server: {}", e))
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}
