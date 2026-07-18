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

//! Interactive Discord UI for selecting one or more PalWorld instances.
//!
//! Mutating commands (announce, kick, ban, etc.) call [`prompt_mutating_target`] before executing.
//! When a tenant has only one instance the function short-circuits without showing any UI.

use poise::serenity_prelude as serenity;
use std::time::Duration;

use crate::models::PalworldInstance;
use crate::{Context, Error};

// ─── InstanceTarget ───────────────────────────────────────────────────────────

/// The server(s) a command should execute against, as chosen by the user.
pub enum InstanceTarget {
    /// Exactly one instance (also used when the tenant has only one enabled instance).
    Single(PalworldInstance),
    /// Two or more instances — either "All Servers" or a multi-select subset.
    Multiple(Vec<PalworldInstance>),
}

impl InstanceTarget {
    /// Iterate over the contained instances regardless of variant.
    #[allow(dead_code)]
    pub fn instances(&self) -> Vec<&PalworldInstance> {
        match self {
            InstanceTarget::Single(i) => vec![i],
            InstanceTarget::Multiple(v) => v.iter().collect(),
        }
    }
}

// ─── Picker ───────────────────────────────────────────────────────────────────

/// Discord API maximum number of options in a select menu component.
const DISCORD_SELECT_MENU_MAX_OPTIONS: usize = 25;

/// Present an ephemeral instance picker and wait for the user to choose.
///
/// * **1 instance** — returns immediately with `InstanceTarget::Single`; no UI shown.
/// * **2–5 instances** — shows a row of buttons: one per server + "All Servers".
/// * **>5 instances** — shows a multi-select menu with an "All Servers" option.
///
/// Returns `None` if the interaction times out.
pub async fn prompt_mutating_target(
    ctx: Context<'_>,
    instances: Vec<PalworldInstance>,
) -> Result<Option<InstanceTarget>, Error> {
    if instances.len() == 1 {
        return Ok(Some(InstanceTarget::Single(
            instances.into_iter().next().unwrap(),
        )));
    }

    // Use the slash command interaction ID as a unique nonce so that concurrent invocations from
    // different users do not accidentally collect each other's button clicks.
    let nonce = ctx.id();
    let all_id = format!("inst:all:{nonce}");
    let inst_ids: Vec<String> = instances
        .iter()
        .map(|i| format!("inst:{}:{nonce}", i.id))
        .collect();

    let prompt_content = "🖥️ **Select target server:**";

    let components: Vec<serenity::CreateActionRow> = if instances.len() <= 5 {
        // Button row: one button per instance + "All Servers"
        let mut buttons: Vec<serenity::CreateButton> = instances
            .iter()
            .zip(inst_ids.iter())
            .map(|(inst, id)| {
                serenity::CreateButton::new(id)
                    .label(&inst.display_name)
                    .style(serenity::ButtonStyle::Primary)
            })
            .collect();
        buttons.push(
            serenity::CreateButton::new(&all_id)
                .label("All Servers")
                .style(serenity::ButtonStyle::Secondary),
        );
        vec![serenity::CreateActionRow::Buttons(buttons)]
    } else {
        // Select menu with multi-select enabled
        let mut options: Vec<serenity::CreateSelectMenuOption> = instances
            .iter()
            .zip(inst_ids.iter())
            .map(|(inst, id)| serenity::CreateSelectMenuOption::new(&inst.display_name, id))
            .collect();
        options.push(serenity::CreateSelectMenuOption::new("All Servers", &all_id));

        let max = (instances.len() + 1).min(DISCORD_SELECT_MENU_MAX_OPTIONS) as u8;
        vec![serenity::CreateActionRow::SelectMenu(
            serenity::CreateSelectMenu::new(
                format!("inst_select:{nonce}"),
                serenity::CreateSelectMenuKind::String { options },
            )
            .min_values(1)
            .max_values(max),
        )]
    };

    let reply = ctx
        .send(
            poise::CreateReply::default()
                .content(prompt_content)
                .components(components)
                .ephemeral(true),
        )
        .await?;

    let message = reply.message().await?;
    let interaction = message
        .await_component_interaction(ctx.serenity_context())
        .timeout(Duration::from_secs(60))
        .await;

    match interaction {
        None => {
            reply
                .edit(
                    ctx,
                    poise::CreateReply::default()
                        .content("⏰ Timed out. Please run the command again.")
                        .components(vec![]),
                )
                .await?;
            Ok(None)
        }
        Some(interaction) => {
            let selected_ids: Vec<String> = match &interaction.data.kind {
                serenity::ComponentInteractionDataKind::Button => {
                    vec![interaction.data.custom_id.clone()]
                }
                serenity::ComponentInteractionDataKind::StringSelect { values } => values.clone(),
                _ => vec![],
            };

            if selected_ids.contains(&all_id) {
                // "All Servers" chosen
                interaction
                    .create_response(
                        ctx.serenity_context(),
                        serenity::CreateInteractionResponse::UpdateMessage(
                            serenity::CreateInteractionResponseMessage::new()
                                .content("▶️ Running on **all servers**…")
                                .components(vec![]),
                        ),
                    )
                    .await?;
                return Ok(Some(InstanceTarget::Multiple(instances)));
            }

            // Collect all matched instances (for multi-select subsets)
            let selected: Vec<PalworldInstance> = instances
                .iter()
                .zip(inst_ids.iter())
                .filter(|(_, id)| selected_ids.contains(id))
                .map(|(inst, _)| inst.clone())
                .collect();

            match selected.len() {
                0 => {
                    interaction
                        .create_response(
                            ctx.serenity_context(),
                            serenity::CreateInteractionResponse::UpdateMessage(
                                serenity::CreateInteractionResponseMessage::new()
                                    .content("❌ Unknown server selection.")
                                    .components(vec![]),
                            ),
                        )
                        .await?;
                    Ok(None)
                }
                1 => {
                    let inst = selected.into_iter().next().unwrap();
                    interaction
                        .create_response(
                            ctx.serenity_context(),
                            serenity::CreateInteractionResponse::UpdateMessage(
                                serenity::CreateInteractionResponseMessage::new()
                                    .content(format!("▶️ Running on **{}**…", inst.display_name))
                                    .components(vec![]),
                            ),
                        )
                        .await?;
                    Ok(Some(InstanceTarget::Single(inst)))
                }
                _ => {
                    let names: Vec<&str> =
                        selected.iter().map(|i| i.display_name.as_str()).collect();
                    interaction
                        .create_response(
                            ctx.serenity_context(),
                            serenity::CreateInteractionResponse::UpdateMessage(
                                serenity::CreateInteractionResponseMessage::new()
                                    .content(format!(
                                        "▶️ Running on **{}**…",
                                        names.join(", ")
                                    ))
                                    .components(vec![]),
                            ),
                        )
                        .await?;
                    Ok(Some(InstanceTarget::Multiple(selected)))
                }
            }
        }
    }
}

// ─── Summary embed builder ────────────────────────────────────────────────────

/// Build a summary embed for operations that ran against multiple instances.
///
/// `results` is a list of `(display_name, outcome_string)` pairs where `outcome_string` starts
/// with `✅` on success or `❌` on failure.
pub fn build_multi_result_embed(
    title: impl Into<String>,
    results: Vec<(String, String)>,
) -> serenity::CreateEmbed {
    let mut embed = serenity::CreateEmbed::new()
        .title(title)
        .color(0x5865f2)
        .timestamp(serenity::Timestamp::now());

    for (name, outcome) in results {
        embed = embed.field(name, outcome, false);
    }

    embed
}

// ─── Server selector by name ─────────────────────────────────────────────────

/// Find a `PalworldInstance` by display name (case-insensitive) from a list.
///
/// Used by read-only commands that accept an optional `server` parameter.
pub fn find_instance_by_name<'a>(
    instances: &'a [PalworldInstance],
    name: &str,
) -> Option<&'a PalworldInstance> {
    let lower = name.to_lowercase();
    instances
        .iter()
        .find(|i| i.display_name.to_lowercase() == lower)
}
