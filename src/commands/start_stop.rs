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

use crate::{Context, Error, ServiceManager};


const PROMPT_TO_REBOOT: &str = "If you need to restart the server, please stop it first using `/stop` and then start it again using `/start`. (**Be careful, this will disconnect all players.**)";


/// Start the server
#[poise::command(slash_command)]
pub async fn start(ctx: Context<'_>) -> Result<(), Error> {

    let data = ctx.data();
    let api_url = &data.palworld_api_url;
    let client = &data.http_client;

    // Check if the server is already running.
    let status_resp = client
        .get(format!("{}/v1/api/info", api_url))
        .basic_auth("admin", Some(&data.admin_password))
        .send()
        .await?;
    
    if status_resp.status().is_success() {
        // It is running, do nothing then.
        ctx.say(
            format!("⚠️ The PalWorld server is already online. {PROMPT_TO_REBOOT}")
        ).await?;
        return Ok(());
    }

    let service_manager = ServiceManager::from_str(&data.palworld_service_manager);
    let service_name = &data.palworld_service_name;

    if !service_manager.is_capable() {
        ctx.say("⚠️ No OS service manager is configured (`palworld_service_manager = \"none\"`). Cannot start the server via a service unit.").await?;
        return Ok(());
    }

    // Attempt to start the server via the configured OS service manager.
    #[cfg(unix)]
    {
        let result = service_manager.start(service_name);
        let exit_code = match result {
            Ok(code) => code,
            Err(e) => {
                ctx.say(format!("❌ Failed to invoke the {} service manager: {}", service_manager.label(), e)).await?;
                return Ok(());
            }
        };

        ctx.send(poise::CreateReply::default().content(
            format!(
                "✅ Initiated PalWorld server via {} (`{}`).\nExit code: {}",
                service_manager.label(),
                service_name,
                exit_code
            )
        )).await?;
    }

    #[cfg(not(unix))]
    {
        ctx.say("⚠️ OS service management is only supported on Unix/Linux platforms.").await?;
    }

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
    let api_url = &data.palworld_api_url;
    let client = &data.http_client;

    let shutdown_time: u64 = time.unwrap_or(60); // Default shutdown time
    let shutdown_message: String = message.unwrap_or_else(|| "initiating shutdown via Discord".to_string());
    // TODO: Limit length of custom message if needed.

    // Check if the server is already stopped.
    let status_resp = client
        .get(format!("{}/v1/api/info", api_url))
        .basic_auth("admin", Some(&data.admin_password))
        .send()
        .await?;
    
    if status_resp.status().is_client_error() {
        // It is stopped, do nothing then.
        ctx.send(
            poise::CreateReply::default().content("⚠️ The PalWorld server is already offline.")
        ).await?;
        return Ok(());
    }

    let status_resp = client
        .post(format!("{}/v1/api/shutdown", api_url))
        .basic_auth("admin", Some(&data.admin_password))
        .body(
            serde_json::json!({
                "waittime": shutdown_time,
                "message": shutdown_message
            }).to_string()
        )
        .send()
        .await?;

    if status_resp.status().is_success() {
        ctx.send(poise::CreateReply::default().content(
            format!("🛑 Sent shutdown command to PalWorld server. Delay: {shutdown_time} seconds.\n  {shutdown_message}"))
        ).await?;
    } else {
        let status = status_resp.status();
        let body = status_resp.text().await.unwrap_or_else(|_| "<failed to read response body>".to_string());
        ctx.send(poise::CreateReply::default().content(
            format!("❌ Failed to send shutdown command to PalWorld server. Status: {status}. Response: {body}")
        )).await?;
    }

    Ok(())

}

/// Force stop the server immediately via the OS service unit (SIGKILL)
#[poise::command(slash_command)]
pub async fn forcestop(ctx: Context<'_>) -> Result<(), Error> {

    let data = ctx.data();
    let api_url = &data.palworld_api_url;
    let client = &data.http_client;

    // Attempt to force stop the server.
    let status_resp = client
        .get(format!("{}/v1/api/info", api_url))
        .basic_auth("admin", Some(&data.admin_password))
        .send()
        .await?;

    if status_resp.status().is_client_error() {
        // It is stopped, do nothing then.
        ctx.send(
            poise::CreateReply::default().content("⚠️ The PalWorld server is already offline.")
        ).await?;
        return Ok(());
    }

    let service_manager = ServiceManager::from_str(&data.palworld_service_manager);
    let service_name = &data.palworld_service_name;

    let manager_note = if service_manager.is_capable() {
        format!(
            "\n> The process will be killed via the **{}** service manager (`{}`).",
            service_manager.label(),
            service_name
        )
    } else {
        "\n> ⚠️ No OS service manager is configured — the PalWorld REST API `/stop` endpoint will be used instead.".to_string()
    };

    // Build a warning button and send it to the user.
    let confirm_button = CreateButton::new("force_stop_confirm")
        .emoji(serenity::ReactionType::Unicode("🔥".to_string()))
        .label("Force Stop")
        .style(serenity::ButtonStyle::Danger);

    let reply = ctx.send(
        poise::CreateReply::default().content(
            format!(
                "🔥 **Warning:** This will immediately terminate the PalWorld server process, which may lead to data loss. All connected players will be immediately disconnected.{manager_note}\n\nAre you sure you want to proceed?"
            )
        ).components(
            vec![serenity::CreateActionRow::Buttons(vec![
                confirm_button
            ])]
        )
    ).await?;

    // Wait for button interaction
    let interaction = reply
        .message()
        .await?
        .await_component_interaction(ctx.serenity_context())
        .timeout(std::time::Duration::from_secs(60)) // 60 second timeout
        .await;

    match interaction {
        Some(interaction) => {
            if interaction.data.custom_id == "force_stop_confirm" {

                // User confirmed, proceed with force stop
                interaction.create_response(
                    ctx.serenity_context(),
                    serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::default()
                            .content("Force stopping the PalWorld server...")
                            .components(vec![]) // Remove the button
                    )
                ).await?;

                let stop_status_message = force_stop_server(client, api_url, &data.admin_password, &service_manager, service_name).await;

                ctx.send(
                    poise::CreateReply::default().content(stop_status_message)
                ).await?;

            }
        }
        None => {
            // Timeout - edit the message to remove the button
            reply.edit(ctx, poise::CreateReply::default()
                .content("⏰ Force stop confirmation timed out. Please run the command again if you still want to force stop the server.")
                .components(vec![])
            ).await?;
        }
    }

    Ok(())

}

/// Performs the actual force-stop, preferring OS service unit kill over the REST API.
async fn force_stop_server(
    client: &reqwest::Client,
    api_url: &str,
    admin_password: &str,
    service_manager: &ServiceManager,
    service_name: &str,
) -> String {
    // Prefer killing the process via the OS service manager when available.
    #[cfg(unix)]
    if service_manager.is_capable() {
        return match service_manager.force_stop(service_name) {
            Ok(code) if code == 0 => {
                format!(
                    "✅ PalWorld server has been force stopped via {} (`{}`).",
                    service_manager.label(),
                    service_name
                )
            }
            Ok(code) => {
                format!(
                    "⚠️ Force stop via {} returned exit code {}. The server may still be running.",
                    service_manager.label(),
                    code
                )
            }
            Err(e) => {
                format!(
                    "❌ Failed to force stop via {}: {}. Falling back to REST API.",
                    service_manager.label(),
                    e
                )
            }
        };
    }

    // Fallback: use the PalWorld REST API /stop endpoint.
    let stop_status = client
        .post(format!("{}/v1/api/stop", api_url))
        .basic_auth("admin", Some(admin_password))
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await;

    match stop_status {
        Ok(s) => format!("✅ PalWorld server has been force stopped via REST API. ({})", s.status()),
        Err(e) => match e.status() {
            Some(status) => format!("❌ Failed to force stop the PalWorld server. (HTTP status: {})", status),
            None => format!("❌ Failed to force stop the PalWorld server. (Error: {})", e),
        },
    }
}

