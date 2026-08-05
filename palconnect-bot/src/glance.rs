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

use log::{debug, error, info, warn};
use poise::serenity_prelude as serenity;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;

use crate::models::PalworldInstance;
use crate::services::TenantStore;
use crate::{BotData, Error};

/// Start the background task that polls PalWorld servers and updates bot status.
pub async fn start_status_updater(
    ctx: Arc<serenity::Context>,
    bot_data: Arc<BotData>,
    update_interval_seconds: u64,
    cancellation_token: CancellationToken,
) -> JoinHandle<()> {
    info!(
        "🔄 Starting status updater with {}s interval",
        update_interval_seconds
    );

    let mut interval_timer = interval(Duration::from_secs(update_interval_seconds));

    // Set initial placeholder status while the first poll runs.
    ctx.set_activity(Some(serenity::ActivityData::playing(
        "/help - PalConnect Commands",
    )));

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    info!("🛑 Status updater shutting down gracefully");
                    break;
                }
                _ = interval_timer.tick() => {
                    match update_bot_status(&ctx, &bot_data).await {
                        Ok(_) => debug!("✅ Status updated successfully"),
                        Err(e) => error!("❌ Failed to update status: {}", e),
                    }
                }
            }
        }
    })
}

/// Poll all tenant instances and update the global Discord presence.
///
/// Global presence is set to aggregate player counts across all tracked instances.
/// Per-guild pinned status messages are also updated for any tenant that has configured a status
/// channel (Phase 3 feature; placeholder logic runs here so the infrastructure is in place).
async fn update_bot_status(ctx: &serenity::Context, bot_data: &BotData) -> Result<(), Error> {
    let store = &*bot_data.tenant_store;
    let client = &*bot_data.palworld_client;

    let tenants = store.get_all_tenants();
    if tenants.is_empty() {
        ctx.set_activity(Some(serenity::ActivityData::playing(
            "No servers configured | /help",
        )));
        return Ok(());
    }

    let mut total_players: usize = 0;
    let mut total_max: usize = 0;
    let mut any_online = false;
    let mut instance_summaries: Vec<(String, bool, usize, usize)> = Vec::new(); // (name, online, current, max)

    for tenant in &tenants {
        if !tenant.enabled {
            continue;
        }
        let instances = store.get_instances_for_tenant(tenant.id);
        let enabled: Vec<&PalworldInstance> = instances.iter().filter(|i| i.enabled).collect();

        for instance in &enabled {
            match client.get_metrics(instance).await {
                Ok(metrics) => {
                    total_players += metrics.current_player_num;
                    total_max += metrics.max_player_num;
                    any_online = true;
                    instance_summaries.push((
                        instance.display_name.clone(),
                        true,
                        metrics.current_player_num,
                        metrics.max_player_num,
                    ));
                    debug!(
                        "📊 {} — {}/{} players",
                        instance.display_name, metrics.current_player_num, metrics.max_player_num
                    );
                }
                Err(e) => {
                    warn!("⚠️ Could not reach {} — {}", instance.display_name, e);
                    instance_summaries.push((instance.display_name.clone(), false, 0, 0));
                }
            }
        }

        // Update per-guild pinned status message if the tenant has one configured.
        if tenant.status_channel_id.is_some() {
            update_tenant_status_message(ctx, bot_data, tenant, &enabled).await;
        }
    }

    // Update global Discord presence with aggregated stats.
    let presence = if any_online {
        let player_word = if total_players == 1 {
            "player"
        } else {
            "players"
        };
        format!(
            "{}/{} {} online | /help",
            total_players, total_max, player_word
        )
    } else {
        "All servers offline | /help".to_string()
    };
    ctx.set_activity(Some(serenity::ActivityData::watching(presence)));

    Ok(())
}

/// Update (or create) the pinned per-guild status message for a tenant.
///
/// Phase 3 implementation: if `status_channel_id` and `status_message_id` are both set, edit the
/// existing message; otherwise post a new one and store its ID.
async fn update_tenant_status_message(
    ctx: &serenity::Context,
    bot_data: &BotData,
    tenant: &crate::models::Tenant,
    instances: &[&PalworldInstance],
) {
    let Some(channel_id) = tenant.status_channel_id else {
        return;
    };
    let channel = serenity::ChannelId::new(channel_id);
    let client = &*bot_data.palworld_client;
    let store = &*bot_data.tenant_store;

    // Build the status text.
    let mut lines: Vec<String> = Vec::new();
    for instance in instances {
        let line = match client.get_metrics(instance).await {
            Ok(m) => format!(
                "🖥️  **{}**   ✅ Online — {}/{} players",
                instance.display_name, m.current_player_num, m.max_player_num
            ),
            Err(_) => format!(
                "🖥️  **{}**   ❌ Offline / unreachable",
                instance.display_name
            ),
        };
        lines.push(line);
    }
    let now = chrono::Utc::now().format("%H:%M:%S UTC");
    lines.push(format!("🔄 *Last updated: {}*", now));
    let content = lines.join("\n");

    if let Some(message_id) = tenant.status_message_id {
        // Edit existing pinned message.
        let msg_id = serenity::MessageId::new(message_id);
        let edit = serenity::EditMessage::new().content(&content);
        if let Err(e) = channel.edit_message(&ctx.http, msg_id, edit).await {
            warn!(
                "⚠️ Could not edit status message for tenant {}: {}",
                tenant.id, e
            );
        }
    } else {
        // Post a new message and pin it.
        match channel.say(&ctx.http, &content).await {
            Ok(msg) => {
                if let Err(e) = channel.pin(&ctx.http, msg.id).await {
                    warn!("⚠️ Could not pin status message: {}", e);
                }
                store.set_tenant_status_channel(tenant.id, Some(channel_id), Some(msg.id.get()));
                info!(
                    "📌 Posted and pinned status message for tenant {}",
                    tenant.id
                );
            }
            Err(e) => {
                warn!(
                    "⚠️ Could not post status message for tenant {}: {}",
                    tenant.id, e
                );
            }
        }
    }
}

/// Manually trigger a status update (useful for the `/update_status` test command).
pub async fn update_status_now(ctx: &serenity::Context, bot_data: &BotData) -> Result<(), Error> {
    update_bot_status(ctx, bot_data).await
}
