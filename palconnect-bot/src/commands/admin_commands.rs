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

use crate::commands::instance_selector::{
    InstanceTarget, build_multi_result_embed, find_instance_by_name, prompt_mutating_target,
};
use crate::services::{resolve_tenant_and_all_instances, resolve_tenant_and_instance};
use crate::utils::sanitize_sensitive_data;
use crate::{Context, Error};

// ─── Read-only commands ───────────────────────────────────────────────────────

/// Print PalWorld server settings
#[poise::command(slash_command)]
pub async fn settings(
    ctx: Context<'_>,
    #[description = "Server name to query (default: primary)"] server: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let data = ctx.data();
    let guild_id = ctx.guild_id().map(|g| g.get());

    let instance = if let Some(ref name) = server {
        let (_tenant, instances) =
            match resolve_tenant_and_all_instances(&*data.tenant_store, guild_id) {
                Ok(pair) => pair,
                Err(msg) => {
                    ctx.send(poise::CreateReply::default().content(msg).ephemeral(true))
                        .await?;
                    return Ok(());
                }
            };
        match find_instance_by_name(&instances, name) {
            Some(i) => i.clone(),
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
        let (_tenant, inst) = match resolve_tenant_and_instance(&*data.tenant_store, guild_id) {
            Ok(pair) => pair,
            Err(msg) => {
                ctx.send(poise::CreateReply::default().content(msg).ephemeral(true))
                    .await?;
                return Ok(());
            }
        };
        inst
    };

    match data.palworld_client.get_settings(&instance).await {
        Ok(raw_settings) => {
            let sanitized = sanitize_sensitive_data(raw_settings);
            ctx.send(
                poise::CreateReply::default().attachment(serenity::CreateAttachment::bytes(
                    serde_json::to_vec_pretty(&sanitized).unwrap_or_else(|err| {
                        format!("Failed to serialize settings: {}", err).into_bytes()
                    }),
                    "palworld_settings.json",
                )),
            )
            .await?;
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

    Ok(())
}

/// Print PalWorld server metrics
#[poise::command(slash_command)]
pub async fn metrics(
    ctx: Context<'_>,
    #[description = "Server name to query (default: primary)"] server: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let data = ctx.data();
    let guild_id = ctx.guild_id().map(|g| g.get());

    let instance = if let Some(ref name) = server {
        let (_tenant, instances) =
            match resolve_tenant_and_all_instances(&*data.tenant_store, guild_id) {
                Ok(pair) => pair,
                Err(msg) => {
                    ctx.send(poise::CreateReply::default().content(msg).ephemeral(true))
                        .await?;
                    return Ok(());
                }
            };
        match find_instance_by_name(&instances, name) {
            Some(i) => i.clone(),
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
        let (_tenant, inst) = match resolve_tenant_and_instance(&*data.tenant_store, guild_id) {
            Ok(pair) => pair,
            Err(msg) => {
                ctx.send(poise::CreateReply::default().content(msg).ephemeral(true))
                    .await?;
                return Ok(());
            }
        };
        inst
    };

    match data.palworld_client.get_metrics(&instance).await {
        Ok(m) => {
            let metrics_text = format!(
                "```\nPlayers:    {}/{}\nUptime:     {}s ({} days)\nServer FPS: {}\nFrame Time: {:.2}ms\n```",
                m.current_player_num,
                m.max_player_num,
                m.uptime,
                m.days,
                m.server_fps,
                m.server_frame_time
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
                    .content(format!(
                        "❌ Failed to reach **{}**: {}",
                        instance.display_name, e
                    ))
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}

// ─── Mutating commands ────────────────────────────────────────────────────────

/// Announce a message to all players
#[poise::command(slash_command)]
pub async fn announce(
    ctx: Context<'_>,
    #[description = "Message to announce to all players"] message: String,
) -> Result<(), Error> {
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

    let target = match prompt_mutating_target(ctx, instances).await? {
        Some(t) => t,
        None => return Ok(()),
    };

    match target {
        InstanceTarget::Single(instance) => {
            ctx.defer().await?;
            match data.palworld_client.announce(&instance, &message).await {
                Ok(()) => {
                    ctx.send(poise::CreateReply::default().content(format!(
                        "📢 Announcement sent to **{}**: \"{}\"",
                        instance.display_name, message
                    )))
                    .await?;
                }
                Err(e) => {
                    ctx.send(
                        poise::CreateReply::default()
                            .content(format!(
                                "❌ Failed to announce on **{}**: {}",
                                instance.display_name, e
                            ))
                            .ephemeral(true),
                    )
                    .await?;
                }
            }
        }
        InstanceTarget::Multiple(instances) => {
            ctx.defer().await?;
            let mut results = Vec::new();
            for instance in &instances {
                let outcome = match data.palworld_client.announce(instance, &message).await {
                    Ok(()) => "✅ Sent".to_string(),
                    Err(e) => format!("❌ {}", e),
                };
                results.push((instance.display_name.clone(), outcome));
            }
            ctx.send(
                poise::CreateReply::default().embed(build_multi_result_embed(
                    format!("📢 Announcement — \"{}\"", message),
                    results,
                )),
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

    let target = match prompt_mutating_target(ctx, instances).await? {
        Some(t) => t,
        None => return Ok(()),
    };

    let kick_message = message.unwrap_or_else(|| "Kicked by admin".to_string());

    match target {
        InstanceTarget::Single(instance) => {
            ctx.defer().await?;
            match data
                .palworld_client
                .kick(&instance, &userid, &kick_message)
                .await
            {
                Ok(()) => {
                    ctx.send(poise::CreateReply::default().content(format!(
                        "👢 Player `{}` has been kicked from **{}**. Reason: \"{}\"",
                        userid, instance.display_name, kick_message
                    )))
                    .await?;
                }
                Err(e) => {
                    ctx.send(
                        poise::CreateReply::default()
                            .content(format!(
                                "❌ Failed to kick on **{}**: {}",
                                instance.display_name, e
                            ))
                            .ephemeral(true),
                    )
                    .await?;
                }
            }
        }
        InstanceTarget::Multiple(instances) => {
            ctx.defer().await?;
            let mut results = Vec::new();
            for instance in &instances {
                let outcome = match data
                    .palworld_client
                    .kick(instance, &userid, &kick_message)
                    .await
                {
                    Ok(()) => "✅ Kicked".to_string(),
                    Err(e) => format!("❌ {}", e),
                };
                results.push((instance.display_name.clone(), outcome));
            }
            ctx.send(
                poise::CreateReply::default().embed(build_multi_result_embed(
                    format!("👢 Kick `{}`", userid),
                    results,
                )),
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

    let target = match prompt_mutating_target(ctx, instances).await? {
        Some(t) => t,
        None => return Ok(()),
    };

    let ban_message = message.unwrap_or_else(|| "Banned by admin".to_string());

    match target {
        InstanceTarget::Single(instance) => {
            ctx.defer().await?;
            match data
                .palworld_client
                .ban(&instance, &userid, &ban_message)
                .await
            {
                Ok(()) => {
                    ctx.send(poise::CreateReply::default().content(format!(
                        "🔨 Player `{}` has been banned from **{}**. Reason: \"{}\"",
                        userid, instance.display_name, ban_message
                    )))
                    .await?;
                }
                Err(e) => {
                    ctx.send(
                        poise::CreateReply::default()
                            .content(format!(
                                "❌ Failed to ban on **{}**: {}",
                                instance.display_name, e
                            ))
                            .ephemeral(true),
                    )
                    .await?;
                }
            }
        }
        InstanceTarget::Multiple(instances) => {
            ctx.defer().await?;
            let mut results = Vec::new();
            for instance in &instances {
                let outcome = match data
                    .palworld_client
                    .ban(instance, &userid, &ban_message)
                    .await
                {
                    Ok(()) => "✅ Banned".to_string(),
                    Err(e) => format!("❌ {}", e),
                };
                results.push((instance.display_name.clone(), outcome));
            }
            ctx.send(
                poise::CreateReply::default().embed(build_multi_result_embed(
                    format!("🔨 Ban `{}`", userid),
                    results,
                )),
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

    let target = match prompt_mutating_target(ctx, instances).await? {
        Some(t) => t,
        None => return Ok(()),
    };

    match target {
        InstanceTarget::Single(instance) => {
            ctx.defer().await?;
            match data.palworld_client.unban(&instance, &userid).await {
                Ok(()) => {
                    ctx.send(poise::CreateReply::default().content(format!(
                        "✅ Player `{}` has been unbanned from **{}**.",
                        userid, instance.display_name
                    )))
                    .await?;
                }
                Err(e) => {
                    ctx.send(
                        poise::CreateReply::default()
                            .content(format!(
                                "❌ Failed to unban on **{}**: {}",
                                instance.display_name, e
                            ))
                            .ephemeral(true),
                    )
                    .await?;
                }
            }
        }
        InstanceTarget::Multiple(instances) => {
            ctx.defer().await?;
            let mut results = Vec::new();
            for instance in &instances {
                let outcome = match data.palworld_client.unban(instance, &userid).await {
                    Ok(()) => "✅ Unbanned".to_string(),
                    Err(e) => format!("❌ {}", e),
                };
                results.push((instance.display_name.clone(), outcome));
            }
            ctx.send(
                poise::CreateReply::default().embed(build_multi_result_embed(
                    format!("✅ Unban `{}`", userid),
                    results,
                )),
            )
            .await?;
        }
    }

    Ok(())
}

/// Save the world
#[poise::command(slash_command)]
pub async fn save(ctx: Context<'_>) -> Result<(), Error> {
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

    let target = match prompt_mutating_target(ctx, instances).await? {
        Some(t) => t,
        None => return Ok(()),
    };

    match target {
        InstanceTarget::Single(instance) => {
            ctx.defer().await?;
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
                            .content(format!(
                                "❌ Failed to save on **{}**: {}",
                                instance.display_name, e
                            ))
                            .ephemeral(true),
                    )
                    .await?;
                }
            }
        }
        InstanceTarget::Multiple(instances) => {
            ctx.defer().await?;
            let mut results = Vec::new();
            for instance in &instances {
                let outcome = match data.palworld_client.save(instance).await {
                    Ok(()) => "✅ Saved".to_string(),
                    Err(e) => format!("❌ {}", e),
                };
                results.push((instance.display_name.clone(), outcome));
            }
            ctx.send(
                poise::CreateReply::default().embed(build_multi_result_embed(
                    "💾 World Save — All Servers",
                    results,
                )),
            )
            .await?;
        }
    }

    Ok(())
}
