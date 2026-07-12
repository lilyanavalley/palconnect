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

use crate::services::resolve_tenant_and_instance;
use crate::{Context, Error};


const PALWORLD_SYSTEMD_NAME: &str = "palworld.service";
const PROMPT_TO_REBOOT: &str = "If you need to restart the server, please stop it first using `/stop` and then start it again using `/start`. (**Be careful, this will disconnect all players.**)";


/// Start the server
#[poise::command(slash_command)]
pub async fn start(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let guild_id = ctx.guild_id().map(|g| g.get());
    let (_tenant, instance) = match resolve_tenant_and_instance(&*data.tenant_store, guild_id) {
        Ok(pair) => pair,
        Err(msg) => {
            ctx.say(msg).await?;
            return Ok(());
        }
    };

    // Check if the server is already running.
    if data.palworld_client.check_online(&instance).await {
        ctx.say(format!("⚠️ **{}** is already online. {PROMPT_TO_REBOOT}", instance.display_name))
            .await?;
        return Ok(());
    }

    // Attempt to start the server via systemd.
    // TODO: Allow choosing how to start the server (systemd, custom script, etc.)
    let process = std::process::Command::new("systemctl")
        .arg("start")
        .arg(PALWORLD_SYSTEMD_NAME) // TODO: Allow custom service name
        .status();

    ctx.send(poise::CreateReply::default().content(format!(
        "✅ Initiated **{}**!\nStatus code: {}",
        instance.display_name,
        process?.code().unwrap_or(-1)
    )))
    .await?;

    Ok(())
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
    let (_tenant, instance) = match resolve_tenant_and_instance(&*data.tenant_store, guild_id) {
        Ok(pair) => pair,
        Err(msg) => {
            ctx.say(msg).await?;
            return Ok(());
        }
    };

    let shutdown_time: u64 = time.unwrap_or(60);
    let shutdown_message = message.unwrap_or_else(|| "initiating shutdown via Discord".to_string());

    // Check if the server is already stopped.
    if !data.palworld_client.check_online(&instance).await {
        ctx.send(
            poise::CreateReply::default()
                .content(format!("⚠️ **{}** is already offline.", instance.display_name)),
        )
        .await?;
        return Ok(());
    }

    match data.palworld_client.shutdown(&instance, shutdown_time, &shutdown_message).await {
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
                    .content(format!("❌ Failed to reach **{}**: {}", instance.display_name, e))
                    .ephemeral(true),
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
    let (_tenant, instance) = match resolve_tenant_and_instance(&*data.tenant_store, guild_id) {
        Ok(pair) => pair,
        Err(msg) => {
            ctx.say(msg).await?;
            return Ok(());
        }
    };

    // Check if the server is already stopped.
    if !data.palworld_client.check_online(&instance).await {
        ctx.send(
            poise::CreateReply::default()
                .content(format!("⚠️ **{}** is already offline.", instance.display_name)),
        )
        .await?;
        return Ok(());
    }

    // Build a warning button and send it to the user.
    let confirm_button = CreateButton::new("force_stop_confirm")
        .emoji(serenity::ReactionType::Unicode("🔥".to_string()))
        .label("Force Stop")
        .style(serenity::ButtonStyle::Danger);

    let reply = ctx.send(
        poise::CreateReply::default()
            .content(format!(
                "🔥 **Warning:** This will immediately terminate **{}**, which may lead to data loss. \
                 All connected players will be immediately disconnected.\n\nAre you sure you want to proceed?",
                instance.display_name
            ))
            .components(vec![serenity::CreateActionRow::Buttons(vec![confirm_button])]),
    )
    .await?;

    // Wait for button interaction
    let interaction = reply
        .message()
        .await?
        .await_component_interaction(ctx.serenity_context())
        .timeout(std::time::Duration::from_secs(60))
        .await;

    match interaction {
        Some(interaction) => {
            if interaction.data.custom_id == "force_stop_confirm" {
                interaction
                    .create_response(
                        ctx.serenity_context(),
                        serenity::CreateInteractionResponse::UpdateMessage(
                            serenity::CreateInteractionResponseMessage::default()
                                .content(format!("Force stopping **{}**…", instance.display_name))
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
                ctx.send(poise::CreateReply::default().content(stop_msg)).await?;
            }
        }
        None => {
            reply
                .edit(
                    ctx,
                    poise::CreateReply::default()
                        .content("⏰ Force stop confirmation timed out. Please run the command again if you still want to force stop the server.")
                        .components(vec![]),
                )
                .await?;
        }
    }

    Ok(())
}

