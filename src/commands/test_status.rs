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

use crate::{Context, Error};
use crate::glance::update_status_now;
use std::sync::Arc;

/// Manually trigger a bot status update (useful for testing)
#[poise::command(slash_command)]
pub async fn update_status(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let bot_data = Arc::new(ctx.data().clone());
    let serenity_ctx = Arc::new(ctx.serenity_context().clone());
    
    match update_status_now(&serenity_ctx, &bot_data).await {
        Ok(_) => {
            ctx.send(poise::CreateReply::default()
                .content("✅ Bot status updated successfully!")
                .ephemeral(true))
                .await?;
        }
        Err(e) => {
            ctx.send(poise::CreateReply::default()
                .content(format!("❌ Failed to update status: {}", e))
                .ephemeral(true))
                .await?;
        }
    }

    Ok(())
}
