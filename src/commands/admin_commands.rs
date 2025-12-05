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
use serde::Deserialize;

use crate::{Context, Error};


#[derive(Debug, Deserialize)]
struct SettingsResponse {
    #[serde(flatten)]
    settings: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct MetricsResponse {
    #[serde(flatten)]
    metrics: serde_json::Value,
}

/// Print PalWorld server settings
#[poise::command(slash_command)]
pub async fn settings(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let data = ctx.data();
    let url = format!("{}/v1/api/settings", data.palworld_api_url);

    match data
        .http_client
        .get(&url)
        .basic_auth("admin", Some(&data.admin_password))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<SettingsResponse>().await {
                    Ok(settings_data) => {

                        // * Admin password is stripped from the output for security reasons.
                        // Even though the REST API doesn't return it, we double-check here in cases where it may be 
                        // returned in the future...
                        let sanitized_settings = sanitize_sensitive_data(settings_data.settings);

                        ctx.send(
                            poise::CreateReply::default().attachment(
                                serenity::CreateAttachment::bytes(
                                    serde_json::to_vec_pretty(&sanitized_settings)
                                        .unwrap_or_else(|err| {
                                            eprintln!("Settings serialization error: {}", err);
                                            format!("Failed to serialize settings: {}", err).into_bytes()
                                        }),
                                    "palworld_settings.json",
                                )
                            )
                        )
                        .await?;

                    }
                    Err(e) => {
                        ctx.send(
                            poise::CreateReply::default()
                                .content(format!("❌ Failed to parse settings response: {}", e))
                                .ephemeral(true),
                        )
                        .await?;
                    }
                }
            } else {
                ctx.send(
                    poise::CreateReply::default()
                        .content(format!("❌ Failed to get settings. Status: {}", response.status()))
                        .ephemeral(true),
                )
                .await?;
            }
        }
        Err(e) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ Failed to connect to PalWorld server: {}", e))
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
    let url = format!("{}/v1/api/metrics", data.palworld_api_url);

    match data
        .http_client
        .get(&url)
        .basic_auth("admin", Some(&data.admin_password))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<MetricsResponse>().await {
                    Ok(metrics_data) => {
                        // Sanitize metrics data as well, in case it contains sensitive information
                        let sanitized_metrics = sanitize_sensitive_data(metrics_data.metrics);
                        
                        // Format metrics as JSON for better readability
                        let metrics_json = serde_json::to_string_pretty(&sanitized_metrics)
                            .unwrap_or_else(|_| "Failed to format metrics".to_string());

                        // Discord has a 1024 character limit for field values
                        let truncated_metrics = if metrics_json.len() > 1000 {
                            format!("{}...\n(truncated)", &metrics_json[..1000])
                        } else {
                            metrics_json
                        };

                        let embed = serenity::CreateEmbed::new()
                            .title("📊 PalWorld Server Metrics")
                            .description(format!("```json\n{}\n```", truncated_metrics))
                            .color(0x00ff00)
                            .timestamp(serenity::Timestamp::now());

                        ctx.send(poise::CreateReply::default().embed(embed)).await?;
                    }
                    Err(e) => {
                        ctx.send(
                            poise::CreateReply::default()
                                .content(format!("❌ Failed to parse metrics response: {}", e))
                                .ephemeral(true),
                        )
                        .await?;
                    }
                }
            } else {
                ctx.send(
                    poise::CreateReply::default()
                        .content(format!("❌ Failed to get metrics. Status: {}", response.status()))
                        .ephemeral(true),
                )
                .await?;
            }
        }
        Err(e) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ Failed to connect to PalWorld server: {}", e))
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
    let url = format!("{}/v1/api/announce", data.palworld_api_url);

    match data
        .http_client
        .post(&url)
        .basic_auth("admin", Some(&data.admin_password))
        .header("Content-Type", "application/json")
        .body(serde_json::json!({ "message": message }).to_string())
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                ctx.send(
                    poise::CreateReply::default()
                        .content(format!("📢 Announcement sent: \"{}\"", message)),
                )
                .await?;
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_else(|_| "<failed to read response body>".to_string());
                ctx.send(
                    poise::CreateReply::default()
                        .content(format!("❌ Failed to send announcement. Status: {}. Response: {}", status, body))
                        .ephemeral(true),
                )
                .await?;
            }
        }
        Err(e) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ Failed to connect to PalWorld server: {}", e))
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
    let url = format!("{}/v1/api/kick", data.palworld_api_url);

    let kick_message = message.unwrap_or_else(|| "Kicked by admin".to_string());

    match data
        .http_client
        .post(&url)
        .basic_auth("admin", Some(&data.admin_password))
        .header("Content-Type", "application/json")
        .body(
            serde_json::json!({
                "userid": userid,
                "message": kick_message
            })
            .to_string(),
        )
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                ctx.send(
                    poise::CreateReply::default()
                        .content(format!("👢 Player `{}` has been kicked. Reason: \"{}\"", userid, kick_message)),
                )
                .await?;
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_else(|_| "<failed to read response body>".to_string());
                ctx.send(
                    poise::CreateReply::default()
                        .content(format!("❌ Failed to kick player. Status: {}. Response: {}", status, body))
                        .ephemeral(true),
                )
                .await?;
            }
        }
        Err(e) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ Failed to connect to PalWorld server: {}", e))
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
    let url = format!("{}/v1/api/ban", data.palworld_api_url);

    let ban_message = message.unwrap_or_else(|| "Banned by admin".to_string());

    match data
        .http_client
        .post(&url)
        .basic_auth("admin", Some(&data.admin_password))
        .header("Content-Type", "application/json")
        .body(
            serde_json::json!({
                "userid": userid,
                "message": ban_message
            })
            .to_string(),
        )
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                ctx.send(
                    poise::CreateReply::default()
                        .content(format!("🔨 Player `{}` has been banned. Reason: \"{}\"", userid, ban_message)),
                )
                .await?;
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_else(|_| "<failed to read response body>".to_string());
                ctx.send(
                    poise::CreateReply::default()
                        .content(format!("❌ Failed to ban player. Status: {}. Response: {}", status, body))
                        .ephemeral(true),
                )
                .await?;
            }
        }
        Err(e) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ Failed to connect to PalWorld server: {}", e))
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
    let url = format!("{}/v1/api/unban", data.palworld_api_url);

    match data
        .http_client
        .post(&url)
        .basic_auth("admin", Some(&data.admin_password))
        .header("Content-Type", "application/json")
        .body(serde_json::json!({ "userid": userid }).to_string())
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                ctx.send(
                    poise::CreateReply::default()
                        .content(format!("✅ Player `{}` has been unbanned.", userid)),
                )
                .await?;
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_else(|_| "<failed to read response body>".to_string());
                ctx.send(
                    poise::CreateReply::default()
                        .content(format!("❌ Failed to unban player. Status: {}. Response: {}", status, body))
                        .ephemeral(true),
                )
                .await?;
            }
        }
        Err(e) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ Failed to connect to PalWorld server: {}", e))
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
    let url = format!("{}/v1/api/save", data.palworld_api_url);

    match data
        .http_client
        .post(&url)
        .basic_auth("admin", Some(&data.admin_password))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                ctx.send(
                    poise::CreateReply::default()
                        .content("💾 World saved successfully!"),
                )
                .await?;
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_else(|_| "<failed to read response body>".to_string());
                ctx.send(
                    poise::CreateReply::default()
                        .content(format!("❌ Failed to save world. Status: {}. Response: {}", status, body))
                        .ephemeral(true),
                )
                .await?;
            }
        }
        Err(e) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("❌ Failed to connect to PalWorld server: {}", e))
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}


/// Recursively sanitize sensitive data from JSON values
fn sanitize_sensitive_data(mut value: serde_json::Value) -> serde_json::Value {
    match &mut value {
        serde_json::Value::Object(map) => {
            // List of sensitive field names to redact
            let sensitive_fields = ["AdminPassword", "admin_password", "password", "secret", "token", "key"];
            
            for (key, val) in map.iter_mut() {
                if sensitive_fields.iter().any(|&field| key.to_lowercase().contains(field)) {
                    *val = serde_json::Value::String("[REDACTED]".to_string());
                } else {
                    *val = sanitize_sensitive_data(val.clone());
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                *item = sanitize_sensitive_data(item.clone());
            }
        }
        _ => {} // Primitives don't need sanitization
    }
    value
}
