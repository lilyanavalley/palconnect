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
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
    let embed = serenity::CreateEmbed::new()
        .title("🤖 PalConnect Bot Help")
        .description("A Discord bot for monitoring your PalWorld dedicated server")
        .field("/players", "Show current online players and count", false)
        .field("/serverinfo", "Display server information", false)
        .field("/start", "Start the PalWorld server", false)
        .field("/stop", "Stop the PalWorld server", false)
        .field("/forcestop", "Force stop the PalWorld server", false)
        .field("/settings", "Print PalWorld server settings", false)
        .field("/metrics", "Print PalWorld server metrics", false)
        .field("/announce", "Announce a message to all players", false)
        .field("/kick", "Kick a player from the server", false)
        .field("/ban", "Ban a player from the server", false)
        .field("/unban", "Unban a previously banned player", false)
        .field("/save", "Save the world", false)
        .field("/help", "Show this help message", false)
        .color(0x7289da)
        .footer(serenity::CreateEmbedFooter::new(concat!(
            "PalConnect Bot ",
            env!("CARGO_PKG_VERSION")
        )));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
