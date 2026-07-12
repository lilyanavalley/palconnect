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
use crate::utils::sanitize_sensitive_data;
use crate::{Context, Error};


/// Print PalWorld server settings
#[poise::command(slash_command)]
pub async fn settings(ctx: Context<'_>) -> Result<(), Error> {
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

    match data.palworld_client.get_settings(&instance).await {
        Ok(raw_settings) => {
            // Sanitize before returning to chat — belt-and-suspenders even though the PalWorld
            // REST API doesn't return admin credentials.
            let sanitized = sanitize_sensitive_data(raw_settings);
            ctx.send(
                poise::CreateReply::default().attachment(
                    serenity::CreateAttachment::bytes(
                        serde_json::to_vec_pretty(&sanitized).unwrap_or_else(|err| {
                            format!("Failed to serialize settings: {}", err).into_bytes()
                        }),
                        "palworld_settings.json",
                    ),
                ),
            )
            .await?;
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

/// Print PalWorld server metrics
#[poise::command(slash_command)]
pub async fn metrics(ctx: Context<'_>) -> Result<(), Error> {
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

    match data.palworld_client.get_metrics(&instance).await {
        Ok(m) => {
            let metrics_text = format!(
                "```\nPlayers:    {}/{}\nUptime:     {}s ({} days)\nServer FPS: {}\nFrame Time: {:.2}ms\n```",
                m.current_player_num, m.max_player_num,
                m.uptime, m.days,
                m.server_fps, m.server_frame_time
            );
            let embed = serenity::CreateEmbed::new()
                .title(format!("📊 {} — Metrics", instance.display_name))
                .description(metrics_text)
                .color(0x00ff00)
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

/// Announce a message to all players
#[poise::command(slash_command)]
pub async fn announce(
    ctx: Context<'_>,
    #[description = "Message to announce to all players"] message: String,
) -> Result<(), Error> {
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

    match data.palworld_client.announce(&instance, &message).await {
        Ok(()) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("📢 Announcement sent to **{}**: \"{}\"", instance.display_name, message)),
            )
            .await?;
        }
        Err(e) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ Failed to announce on **{}**: {}", instance.display_name, e))
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}

/// Kick a player from the server
#[poise::command(slash_command)]
pub async fn kick(
    ctx: Context<'_>,
    #[description = "Player ID or Steam ID to kick"] userid: String,
    #[description = "Reason for kicking (optional)"] message: Option<String>,
) -> Result<(), Error> {
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

    let kick_message = message.unwrap_or_else(|| "Kicked by admin".to_string());

    match data.palworld_client.kick(&instance, &userid, &kick_message).await {
        Ok(()) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("👢 Player `{}` has been kicked from **{}**. Reason: \"{}\"", userid, instance.display_name, kick_message)),
            )
            .await?;
        }
        Err(e) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ Failed to kick on **{}**: {}", instance.display_name, e))
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}

/// Ban a player from the server
#[poise::command(slash_command)]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "Player ID or Steam ID to ban"] userid: String,
    #[description = "Reason for banning (optional)"] message: Option<String>,
) -> Result<(), Error> {
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

    let ban_message = message.unwrap_or_else(|| "Banned by admin".to_string());

    match data.palworld_client.ban(&instance, &userid, &ban_message).await {
        Ok(()) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("🔨 Player `{}` has been banned from **{}**. Reason: \"{}\"", userid, instance.display_name, ban_message)),
            )
            .await?;
        }
        Err(e) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ Failed to ban on **{}**: {}", instance.display_name, e))
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}

/// Unban a previously banned player
#[poise::command(slash_command)]
pub async fn unban(
    ctx: Context<'_>,
    #[description = "Player ID or Steam ID to unban"] userid: String,
) -> Result<(), Error> {
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

    match data.palworld_client.unban(&instance, &userid).await {
        Ok(()) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("✅ Player `{}` has been unbanned from **{}**.", userid, instance.display_name)),
            )
            .await?;
        }
        Err(e) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ Failed to unban on **{}**: {}", instance.display_name, e))
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}

/// Save the world
#[poise::command(slash_command)]
pub async fn save(ctx: Context<'_>) -> Result<(), Error> {
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

    match data.palworld_client.save(&instance).await {
        Ok(()) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("💾 World saved on **{}**!", instance.display_name)),
            )
            .await?;
        }
        Err(e) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ Failed to save on **{}**: {}", instance.display_name, e))
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}

