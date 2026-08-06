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
use poise::serenity_prelude::Mentionable;

use crate::glance::update_status_now;
use crate::services::TenantStore;
use crate::{Context, Error};

const ERR_NO_GUILD: &str = "⚠️ This command must be used inside a Discord server.";

/// Set the channel where PalConnect posts a live pinned server-status message.
///
/// After setting the channel, the bot immediately posts and pins the status message.
/// On every subsequent poll the message is edited in place so the channel stays clean.
/// Use `/clear_status_channel` to stop posting status updates.
#[poise::command(slash_command, required_permissions = "MANAGE_CHANNELS")]
pub async fn set_status_channel(
    ctx: Context<'_>,
    #[description = "Channel to post the live status message in"] channel: serenity::GuildChannel,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let guild_id = match ctx.guild_id() {
        Some(id) => id.get(),
        None => {
            ctx.send(
                poise::CreateReply::default()
                    .content(ERR_NO_GUILD)
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    let data = ctx.data();
    let store = &*data.tenant_store;

    // Resolve tenant so we know the correct tenant ID.
    let tenant = match store.get_tenant_for_guild(Some(guild_id)) {
        Some(t) => t,
        None => {
            ctx.send(
                poise::CreateReply::default()
                    .content("⚠️ This server has not been configured yet. Please contact your administrator.")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    // Clear any existing pinned-message ID so the poller posts a fresh one in the new channel.
    store.set_tenant_status_channel(tenant.id, Some(channel.id.get()), None);

    // Trigger an immediate status update so the message appears right away.
    let serenity_ctx = ctx.serenity_context();
    if let Err(e) = update_status_now(serenity_ctx, data).await {
        ctx.send(
            poise::CreateReply::default()
                .content(format!(
                    "✅ Status channel set to {}, but the initial post failed: {}",
                    channel.mention(),
                    e
                ))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    ctx.send(
        poise::CreateReply::default()
            .content(format!(
                "📌 Status channel set to {}. The live status message has been posted and pinned.",
                channel.mention()
            ))
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

/// Stop posting the live server-status message and unpin it from its channel.
///
/// After running this command, no further automatic status messages will be posted
/// for this server.  Use `/set_status_channel` to re-enable status updates.
#[poise::command(slash_command, required_permissions = "MANAGE_CHANNELS")]
pub async fn clear_status_channel(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let guild_id = match ctx.guild_id() {
        Some(id) => id.get(),
        None => {
            ctx.send(
                poise::CreateReply::default()
                    .content(ERR_NO_GUILD)
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    let data = ctx.data();
    let store = &*data.tenant_store;

    let tenant = match store.get_tenant_for_guild(Some(guild_id)) {
        Some(t) => t,
        None => {
            ctx.send(
                poise::CreateReply::default()
                    .content("⚠️ This server has not been configured yet.")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    let had_channel = tenant.status_channel_id.is_some();
    let channel_id = tenant.status_channel_id;
    let message_id = tenant.status_message_id;

    // Clear the status channel configuration.
    store.set_tenant_status_channel(tenant.id, None, None);

    // Attempt to unpin and delete the existing status message, best-effort.
    if let (Some(ch_id), Some(msg_id)) = (channel_id, message_id) {
        let channel = serenity::ChannelId::new(ch_id);
        let msg = serenity::MessageId::new(msg_id);
        let http = &ctx.serenity_context().http;
        let _ = channel.unpin(http, msg).await;
        let _ = channel.delete_message(http, msg).await;
    }

    let reply = if had_channel {
        "✅ Status channel cleared. No further automatic status messages will be posted."
    } else {
        "ℹ️ No status channel was configured."
    };

    ctx.send(
        poise::CreateReply::default()
            .content(reply)
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
