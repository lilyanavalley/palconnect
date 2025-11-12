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

use crate::{Context, Error};


// PalWorld API response structures
#[derive(Debug, Deserialize)]
struct ServerInfo {
    version: String,
    servername: String,
    description: String,
}

/// Show server information
#[poise::command(slash_command)]
pub async fn serverinfo(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let data = ctx.data();
    let url = format!("{}/v1/api/info", data.palworld_api_url);

    match data
        .http_client
        .get(&url)
        .basic_auth("admin", Some(&data.admin_password))
        .send()
        .await
    {
        Ok(response) => match response.json::<ServerInfo>().await {
            Ok(server_info) => {
                let embed = serenity::CreateEmbed::new()
                    // TODO: Allow custom server info by selections
                    .title("🏰 Server Information")
                    .field("Server Name", &server_info.servername, true)
                    .field("Version", &server_info.version, true)
                    .field("Description", &server_info.description, false)
                    .color(0x0099ff) // TODO: Allow custom color
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
