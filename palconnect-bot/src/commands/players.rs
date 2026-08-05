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

use crate::commands::instance_selector::find_instance_by_name;
use crate::services::resolve_tenant_and_all_instances;
use crate::{Context, Error};

/// Show current players on the PalWorld server(s).
///
/// With a single configured server the output matches the pre-Phase-2 behaviour.
/// With multiple servers the list is aggregated across all of them unless `server` is specified.
#[poise::command(slash_command)]
pub async fn players(
    ctx: Context<'_>,
    #[description = "Server name to query (default: all servers)"] server: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let data = ctx.data();
    let guild_id = ctx.guild_id().map(|g| g.get());
    let (_tenant, instances) = match resolve_tenant_and_all_instances(&*data.tenant_store, guild_id)
    {
        Ok(pair) => pair,
        Err(msg) => {
            ctx.send(poise::CreateReply::default().content(msg).ephemeral(true))
                .await?;
            return Ok(());
        }
    };

    // Select the target instance(s).
    if instances.len() == 1 || server.is_some() {
        // Single target: either only one server, or the user explicitly picked one.
        let instance = if let Some(ref name) = server {
            match find_instance_by_name(&instances, name) {
                Some(i) => i,
                None => {
                    ctx.send(
                        poise::CreateReply::default()
                            .content(format!("❌ No server named **{}** found.", name))
                            .ephemeral(true),
                    )
                    .await?;
                    return Ok(());
                }
            }
        } else {
            &instances[0]
        };

        match data.palworld_client.get_players(instance).await {
            Ok(players_data) => {
                let count = players_data.players.len();
                let list = if players_data.players.is_empty() {
                    "No players currently online".to_string()
                } else {
                    players_data
                        .players
                        .iter()
                        .map(|p| format!("• {} (Level {})", p.name, p.level))
                        .collect::<Vec<_>>()
                        .join("\n")
                };

                let embed = serenity::CreateEmbed::new()
                    .title(format!("🎮 {} — Players", instance.display_name))
                    .field("Online", count.to_string(), true)
                    .field("Player List", list, false)
                    .color(if count > 0 { 0x00ff00 } else { 0xff0000 })
                    .timestamp(serenity::Timestamp::now());

                ctx.send(poise::CreateReply::default().embed(embed)).await?;
            }
            Err(e) => {
                ctx.send(
                    poise::CreateReply::default()
                        .content(format!(
                            "❌ Failed to reach **{}**: {}",
                            instance.display_name, e
                        ))
                        .ephemeral(true),
                )
                .await?;
            }
        }
    } else {
        // Multiple servers: aggregate player lists across all instances.
        let mut embed = serenity::CreateEmbed::new()
            .title("🎮 Players — All Servers")
            .color(0x5865f2)
            .timestamp(serenity::Timestamp::now());

        let mut total = 0usize;

        for instance in &instances {
            match data.palworld_client.get_players(instance).await {
                Ok(players_data) => {
                    let count = players_data.players.len();
                    total += count;
                    let list = if players_data.players.is_empty() {
                        "*No players online*".to_string()
                    } else {
                        players_data
                            .players
                            .iter()
                            .map(|p| format!("• {} (Level {})", p.name, p.level))
                            .collect::<Vec<_>>()
                            .join("\n")
                    };
                    embed = embed.field(
                        format!("🖥️ {} — {} player(s)", instance.display_name, count),
                        list,
                        false,
                    );
                }
                Err(e) => {
                    embed = embed.field(
                        format!("🖥️ {} — ❌ unreachable", instance.display_name),
                        format!("{}", e),
                        false,
                    );
                }
            }
        }

        embed = embed.description(format!("**Total online: {}**", total));
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
    }

    Ok(())
}
