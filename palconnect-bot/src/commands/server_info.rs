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

/// Show server information.
///
/// With a single configured server the output matches the pre-Phase-2 behaviour.
/// With multiple servers information for each instance is shown unless `server` is specified.
#[poise::command(slash_command)]
pub async fn serverinfo(
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

    if instances.len() == 1 || server.is_some() {
        // Single target
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

        match data.palworld_client.get_server_info(instance).await {
            Ok(info) => {
                let embed = serenity::CreateEmbed::new()
                    .title("🏰 Server Information")
                    .field("Server Name", &info.servername, true)
                    .field("Version", &info.version, true)
                    .field("Description", &info.description, false)
                    .color(0x0099ff)
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
        // Multiple servers: one embed section per instance.
        let mut embed = serenity::CreateEmbed::new()
            .title("🏰 Server Information — All Servers")
            .color(0x0099ff)
            .timestamp(serenity::Timestamp::now());

        for instance in &instances {
            match data.palworld_client.get_server_info(instance).await {
                Ok(info) => {
                    embed = embed.field(
                        format!("🖥️ {}", instance.display_name),
                        format!(
                            "**Name:** {}\n**Version:** {}\n**Description:** {}",
                            info.servername, info.version, info.description
                        ),
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

        ctx.send(poise::CreateReply::default().embed(embed)).await?;
    }

    Ok(())
}
