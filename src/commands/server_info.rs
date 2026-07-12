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


/// Show server information
#[poise::command(slash_command)]
pub async fn serverinfo(ctx: Context<'_>) -> Result<(), Error> {
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

    match data.palworld_client.get_server_info(&instance).await {
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
                    .content(format!("❌ Failed to reach **{}**: {}", instance.display_name, e))
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}

