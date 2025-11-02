
use poise::serenity_prelude as serenity;
use serde::Deserialize;

use crate::{Context, Error};


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

/// Show current player count on the PalWorld server
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
                let player_count = players_data.players.len();
                let player_list = if players_data.players.is_empty() {
                    "No players currently online".to_string()
                } else {
                    players_data
                        .players
                        .iter()
                        .map(|p| format!("• {} (Level {})", p.name, p.level))
                        .collect::<Vec<String>>()
                        .join("\n")
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
