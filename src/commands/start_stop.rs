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

use poise::serenity_prelude::{self as serenity, CreateButton};

use crate::commands::instance_selector::{
    build_multi_result_embed, prompt_mutating_target, InstanceTarget,
};
use crate::models::PalworldInstance;
use crate::services::resolve_tenant_and_all_instances;
use crate::{Context, Error};


const PALWORLD_SYSTEMD_NAME: &str = "palworld.service";
const PROMPT_TO_REBOOT: &str = "If you need to restart the server, please stop it first using `/stop` and then start it again using `/start`. (**Be careful, this will disconnect all players.**)";


/// Start the server
#[poise::command(slash_command)]
pub async fn start(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let guild_id = ctx.guild_id().map(|g| g.get());
    let (_tenant, instances) =
        match resolve_tenant_and_all_instances(&*data.tenant_store, guild_id) {
            Ok(pair) => pair,
            Err(msg) => {
                ctx.say(msg).await?;
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
            start_single(ctx, &instance).await?;
        }
        InstanceTarget::Multiple(instances) => {
            ctx.defer().await?;
            let mut results = Vec::new();
            for instance in &instances {
                let outcome = start_instance(instance).await;
                results.push((instance.display_name.clone(), outcome));
            }
            ctx.send(
                poise::CreateReply::default()
                    .embed(build_multi_result_embed("▶️ Start — All Servers", results)),
            )
            .await?;
        }
    }

    Ok(())
}

/// Attempt to start a single instance via systemd and send the result as a reply.
async fn start_single(ctx: Context<'_>, instance: &PalworldInstance) -> Result<(), Error> {
    if ctx.data().palworld_client.check_online(instance).await {
        ctx.say(format!(
            "⚠️ **{}** is already online. {PROMPT_TO_REBOOT}",
            instance.display_name
        ))
        .await?;
        return Ok(());
    }

    let outcome = start_instance(instance).await;
    ctx.send(poise::CreateReply::default().content(format!(
        "**{}**: {}",
        instance.display_name, outcome
    )))
    .await?;
    Ok(())
}

/// Run `systemctl start` for an instance.
///
/// Returns a human-readable outcome string (✅ / ❌).
///
/// # Note
/// The systemd service name is currently hardcoded to `palworld.service` for all instances.
/// Per-instance service name configuration is tracked as a TODO for a future release.
#[allow(unused_variables)]
async fn start_instance(instance: &PalworldInstance) -> String {
    // TODO: Allow custom service name per instance (use instance.display_name or a new field)
    let process = std::process::Command::new("systemctl")
        .arg("start")
        .arg(PALWORLD_SYSTEMD_NAME)
        .status();

    match process {
        Ok(status) => format!(
            "✅ Initiated (exit code: {})",
            status.code().unwrap_or(-1)
        ),
        Err(e) => format!("❌ Failed to invoke systemctl: {}", e),
    }
}

// TODO: Take shortened arguments for time (-t) and message (-m).
/// Stop the server. May take 2 arguments: --time <seconds> / --message <custom shutdown message>
#[poise::command(slash_command)]
pub async fn stop(
    ctx: Context<'_>,
    #[description = "Shutdown delay in seconds (default: 60)"] time: Option<u64>,
    #[description = "Custom shutdown message"] message: Option<String>,
) -> Result<(), Error> {
    let data = ctx.data();
    let guild_id = ctx.guild_id().map(|g| g.get());
    let (_tenant, instances) =
        match resolve_tenant_and_all_instances(&*data.tenant_store, guild_id) {
            Ok(pair) => pair,
            Err(msg) => {
                ctx.say(msg).await?;
                return Ok(());
            }
        };

    let target = match prompt_mutating_target(ctx, instances).await? {
        Some(t) => t,
        None => return Ok(()),
    };

    let shutdown_time = time.unwrap_or(60);
    let shutdown_message = message.unwrap_or_else(|| "initiating shutdown via Discord".to_string());

    match target {
        InstanceTarget::Single(instance) => {
            ctx.defer().await?;
            if !data.palworld_client.check_online(&instance).await {
                ctx.send(
                    poise::CreateReply::default().content(format!(
                        "⚠️ **{}** is already offline.",
                        instance.display_name
                    )),
                )
                .await?;
                return Ok(());
            }

            match data
                .palworld_client
                .shutdown(&instance, shutdown_time, &shutdown_message)
                .await
            {
                Ok(status) if status.is_success() => {
                    ctx.send(poise::CreateReply::default().content(format!(
                        "🛑 Sent shutdown command to **{}**. Delay: {} seconds.\n  {}",
                        instance.display_name, shutdown_time, shutdown_message
                    )))
                    .await?;
                }
                Ok(status) => {
                    ctx.send(poise::CreateReply::default().content(format!(
                        "❌ Shutdown command rejected by **{}**. Status: {}",
                        instance.display_name, status
                    )))
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
        }
        InstanceTarget::Multiple(instances) => {
            ctx.defer().await?;
            let mut results = Vec::new();
            for instance in &instances {
                let outcome = if !data.palworld_client.check_online(instance).await {
                    "⚠️ Already offline".to_string()
                } else {
                    match data
                        .palworld_client
                        .shutdown(instance, shutdown_time, &shutdown_message)
                        .await
                    {
                        Ok(s) if s.is_success() => format!(
                            "✅ Shutdown in {}s",
                            shutdown_time
                        ),
                        Ok(s) => format!("❌ Rejected ({})", s),
                        Err(e) => format!("❌ {}", e),
                    }
                };
                results.push((instance.display_name.clone(), outcome));
            }
            ctx.send(
                poise::CreateReply::default()
                    .embed(build_multi_result_embed("🛑 Stop — All Servers", results)),
            )
            .await?;
        }
    }

    Ok(())
}

/// Force stop the server
#[poise::command(slash_command)]
pub async fn forcestop(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let guild_id = ctx.guild_id().map(|g| g.get());
    let (_tenant, instances) =
        match resolve_tenant_and_all_instances(&*data.tenant_store, guild_id) {
            Ok(pair) => pair,
            Err(msg) => {
                ctx.say(msg).await?;
                return Ok(());
            }
        };

    if instances.len() == 1 {
        // Single instance: show the confirmation button as before.
        let instance = instances.into_iter().next().unwrap();
        force_stop_single(ctx, instance).await
    } else {
        // Multiple instances: the instance picker buttons serve as the confirmation.
        // The buttons are styled Danger and the prompt makes the destructive nature explicit.
        let nonce = ctx.id();
        let all_id = format!("fs:all:{nonce}");
        let inst_ids: Vec<String> = instances
            .iter()
            .map(|i| format!("fs:{}:{nonce}", i.id))
            .collect();

        let buttons: Vec<serenity::CreateButton> = instances
            .iter()
            .zip(inst_ids.iter())
            .map(|(inst, id)| {
                serenity::CreateButton::new(id)
                    .label(format!("🔥 {}", inst.display_name))
                    .style(serenity::ButtonStyle::Danger)
            })
            .chain(std::iter::once(
                serenity::CreateButton::new(&all_id)
                    .label("🔥 All Servers")
                    .style(serenity::ButtonStyle::Danger),
            ))
            .collect();

        let reply = ctx
            .send(
                poise::CreateReply::default()
                    .content(
                        "🔥 **Force stop** will immediately kill the selected server(s), \
                         potentially causing data loss.\n\
                         All connected players will be disconnected instantly.\n\n\
                         **Select a server to force stop:**",
                    )
                    .components(vec![serenity::CreateActionRow::Buttons(buttons)]),
            )
            .await?;

        let message = reply.message().await?;
        let interaction = message
            .await_component_interaction(ctx.serenity_context())
            .timeout(std::time::Duration::from_secs(60))
            .await;

        match interaction {
            None => {
                reply
                    .edit(
                        ctx,
                        poise::CreateReply::default()
                            .content("⏰ Force stop confirmation timed out. Please run the command again.")
                            .components(vec![]),
                    )
                    .await?;
            }
            Some(interaction) => {
                let selected_id = interaction.data.custom_id.clone();

                let target_instances: Vec<_> = if selected_id == all_id {
                    instances
                } else {
                    instances
                        .into_iter()
                        .zip(inst_ids.iter())
                        .filter(|(_, id)| selected_id == **id)
                        .map(|(inst, _)| inst)
                        .collect()
                };

                let names: Vec<&str> =
                    target_instances.iter().map(|i| i.display_name.as_str()).collect();
                interaction
                    .create_response(
                        ctx.serenity_context(),
                        serenity::CreateInteractionResponse::UpdateMessage(
                            serenity::CreateInteractionResponseMessage::new()
                                .content(format!("Force stopping **{}**…", names.join(", ")))
                                .components(vec![]),
                        ),
                    )
                    .await?;

                let mut results = Vec::new();
                for instance in &target_instances {
                    let outcome =
                        match data.palworld_client.force_stop(instance).await {
                            Ok(status) => format!("✅ Force stopped ({})", status),
                            Err(e) => format!("❌ {}", e),
                        };
                    results.push((instance.display_name.clone(), outcome));
                }

                ctx.send(
                    poise::CreateReply::default().embed(build_multi_result_embed(
                        "🔥 Force Stop Results",
                        results,
                    )),
                )
                .await?;
            }
        }

        Ok(())
    }
}

/// Handle the single-instance force-stop flow (confirmation button then execute).
async fn force_stop_single(ctx: Context<'_>, instance: PalworldInstance) -> Result<(), Error> {
    let data = ctx.data();

    if !data.palworld_client.check_online(&instance).await {
        ctx.send(
            poise::CreateReply::default()
                .content(format!("⚠️ **{}** is already offline.", instance.display_name)),
        )
        .await?;
        return Ok(());
    }

    let confirm_button = CreateButton::new("force_stop_confirm")
        .emoji(serenity::ReactionType::Unicode("🔥".to_string()))
        .label("Force Stop")
        .style(serenity::ButtonStyle::Danger);

    let reply = ctx
        .send(
            poise::CreateReply::default()
                .content(format!(
                    "🔥 **Warning:** This will immediately terminate **{}**, which may lead to data loss. \
                     All connected players will be immediately disconnected.\n\nAre you sure you want to proceed?",
                    instance.display_name
                ))
                .components(vec![serenity::CreateActionRow::Buttons(vec![
                    confirm_button,
                ])]),
        )
        .await?;

    let interaction = reply
        .message()
        .await?
        .await_component_interaction(ctx.serenity_context())
        .timeout(std::time::Duration::from_secs(60))
        .await;

    match interaction {
        Some(interaction) if interaction.data.custom_id == "force_stop_confirm" => {
            interaction
                .create_response(
                    ctx.serenity_context(),
                    serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::default()
                            .content(format!(
                                "Force stopping **{}**…",
                                instance.display_name
                            ))
                            .components(vec![]),
                    ),
                )
                .await?;

            let stop_msg = match data.palworld_client.force_stop(&instance).await {
                Ok(status) => format!(
                    "✅ **{}** has been force stopped. ({})",
                    instance.display_name, status
                ),
                Err(e) => format!(
                    "❌ Failed to force stop **{}**. ({})",
                    instance.display_name, e
                ),
            };
            ctx.send(poise::CreateReply::default().content(stop_msg))
                .await?;
        }
        _ => {
            reply
                .edit(
                    ctx,
                    poise::CreateReply::default()
                        .content(
                            "⏰ Force stop confirmation timed out. Please run the command again \
                             if you still want to force stop the server.",
                        )
                        .components(vec![]),
                )
                .await?;
        }
    }

    Ok(())
}
