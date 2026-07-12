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

use crate::services::resolve_tenant_and_instance;
use crate::{Context, Error};


/// Show current player count on the PalWorld server
#[poise::command(slash_command)]
pub async fn players(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let data = ctx.data();
    let guild_id = ctx.guild_id().map(|g| g.get());
    let (_tenant, instance) = match resolve_tenant_and_instance(&*data.tenant_store, guild_id) {
        Ok(pair) => pair,
        Err(msg) => {
            ctx.send(poise::CreateReply::default().content(msg).ephemeral(true)).await?;
            return Ok(());
        }
    };

    match data.palworld_client.get_players(&instance).await {
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
                .title(format!("🎮 {} — Player Status", instance.display_name))
                .field("Players Online", player_count.to_string(), true)
                .field("Player List", player_list, false)
                .color(if player_count > 0 { 0x00ff00 } else { 0xff0000 })
                .timestamp(serenity::Timestamp::now());

            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ Failed to reach **{}**: {}", instance.display_name, e))
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}

