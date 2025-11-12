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

use crate::{Context, Error};


/// Show help information
#[poise::command(slash_command)]
pub async fn about(ctx: Context<'_>) -> Result<(), Error> {
    let embed = serenity::CreateEmbed::new()
        .title("ℹ️ About PalConnect")
        .description("PalConnect is a Discord bot for monitoring your PalWorld dedicated server and integrating server management commands into Discord.")
        .field("Version", env!("CARGO_PKG_VERSION"), false)
        .field("Author", "Lily Ana Valley — <hi@lilyvalley.dev>", false)
        .field("License", "GNU Affero General Public License v3.0 (AGPLv3) https://www.gnu.org/licenses/agpl-3.0.en.html", false)
        .field("Source Code", "https://github.com/lilyanavalley/palconnect", false)
        .color(0x7289da);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
