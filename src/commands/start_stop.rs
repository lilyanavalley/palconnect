
use poise::serenity_prelude::{self as serenity, CreateButton};
use serde::Deserialize;

use crate::{Context, Error};


const PALWORLD_SYSTEMD_NAME: &str = "palworld.service";
const PROMPT_TO_REBOOT: &str = "Type `/restart` to restart the server instead (**be careful, this will disconnect all players.**)";


/// Start the server
#[poise::command(slash_command)]
pub async fn start(ctx: Context<'_>) -> Result<(), Error> {

    let data = ctx.data();
    let api_url = &data.palworld_api_url;
    let client = &data.http_client;

    // Check if the server is already running.
    let status_resp = client
        .get(format!("{}/v1/api/info", api_url))
        .send()
        .await?;
    
    if status_resp.status().is_success() {
        // It is running, do nothing then.
        ctx.say(
            format!("⚠️ The PalWorld server is already online. {PROMPT_TO_REBOOT}")
        ).await?;
        return Ok(());
    }

    // Attempt to start the server.
    // TODO: Allow choosing how to start the server (systemd, custom script, etc.)
    let process = std::process::Command::new("systemctl")
        .arg("start")
        .arg(PALWORLD_SYSTEMD_NAME) // TODO: Allow custom service name
        .status();

    ctx.send(poise::CreateReply::default().content(
        format!("✅ Initiated PalWorld server!\nStatus code: {}", process?.code().unwrap_or(-1)
    ))).await?;

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
    let mut shutdown_message: String = message.unwrap_or_else(|| "initiating shutdown via Discord".to_string());
    // TODO: Limit length of custom message if needed.

    // Check if the server is already stopped.
    let status_resp = client
        .get(format!("{}/v1/api/info", api_url))
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
        .body(format!(
            r#"{{"waittime": {},"message": "{}"}}"#,
            shutdown_time, shutdown_message
        ))
        .send()
        .await?;

    if status_resp.status().is_success() {
        ctx.send(poise::CreateReply::default().content(
            format!("🛑 Sent shutdown command to PalWorld server. Delay: {shutdown_time} seconds.\n  {shutdown_message}"))
        ).await?;
    }

    Ok(())

}

/// Force stop the server
#[poise::command(slash_command)]
pub async fn force_stop(ctx: Context<'_>) -> Result<(), Error> {

    let data = ctx.data();
    let api_url = &data.palworld_api_url;
    let client = &data.http_client;

    // Attempt to force stop the server.
    let status_resp = client
        .get(format!("{}/v1/api/info", api_url))
        .send()
        .await?;

    if status_resp.status().is_client_error() {
        // It is stopped, do nothing then.
        ctx.send(
            poise::CreateReply::default().content("⚠️ The PalWorld server is already offline.")
        ).await?;
        return Ok(());
    }

    // Build a warning button and send it to the user.
    let confirm_button = CreateButton::new("force_stop_confirm")
        .emoji(serenity::ReactionType::Unicode("🔥".to_string()))
        .label("Force Stop")
        .style(serenity::ButtonStyle::Danger);

    let reply = ctx.send(
        poise::CreateReply::default().content(
            "🔥 **Warning:** This will immediately terminate the PalWorld server process, which may lead to data loss. All connected players will be immediately disconnected.\n\nAre you sure you want to proceed?"
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

                let stop_status = client
                    .post(format!("{}/v1/api/stop", api_url))
                    .timeout(std::time::Duration::from_secs(3))
                    .send()
                    .await;

                let stop_status_message = match stop_status {
                    Ok(s) => format!("✅ PalWorld server has been force stopped. ({})", s.status()),
                    Err(e) => match e.status() {
                        Some(status) => format!("❌ Failed to force stop the PalWorld server. (HTTP status: {})", status),
                        None => format!("❌ Failed to force stop the PalWorld server. (Error: {})", e.to_string()),
                    },
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
