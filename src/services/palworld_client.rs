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

use reqwest::{Client, StatusCode};
use serde::Deserialize;
use std::time::Duration;

use crate::models::PalworldInstance;

type Error = Box<dyn std::error::Error + Send + Sync>;

// ─── API response types ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct PlayerInfo {
    pub name: String,
    #[serde(rename = "playerId")]
    pub player_id: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub ip: String,
    pub ping: f64,
    pub location_x: f64,
    pub location_y: f64,
    pub level: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PlayersResponse {
    pub players: Vec<PlayerInfo>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerInfoResponse {
    pub version: String,
    pub servername: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetricsResponse {
    #[serde(rename = "currentplayernum")]
    pub current_player_num: usize,
    #[serde(rename = "maxplayernum")]
    pub max_player_num: usize,
    #[serde(rename = "serverframetime")]
    pub server_frame_time: f32,
    #[serde(rename = "serverfps")]
    pub server_fps: usize,
    #[serde(rename = "uptime")]
    pub uptime: usize,
    #[serde(rename = "days")]
    pub days: usize,
}

// ─── Client ──────────────────────────────────────────────────────────────────

const DEFAULT_TIMEOUT_SECS: u64 = 10;

/// HTTP client for the PalWorld Dedicated Server REST API.
///
/// All methods accept a `&PalworldInstance` so credentials are resolved per-call and never stored
/// in the client itself.  Credentials are never surfaced in error messages or return values.
#[derive(Clone)]
pub struct PalworldClient {
    http: Client,
}

impl PalworldClient {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
        }
    }

    fn url(instance: &PalworldInstance, path: &str) -> String {
        format!("{}{}", instance.api_url.trim_end_matches('/'), path)
    }

    // ─── Read operations ──────────────────────────────────────────────────

    /// Returns `true` when the server responds successfully to an info request.
    pub async fn check_online(&self, instance: &PalworldInstance) -> bool {
        self.http
            .get(Self::url(instance, "/v1/api/info"))
            .basic_auth("admin", Some(&instance.admin_password))
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// Fetch current player list.
    pub async fn get_players(&self, instance: &PalworldInstance) -> Result<PlayersResponse, Error> {
        let resp = self
            .http
            .get(Self::url(instance, "/v1/api/players"))
            .basic_auth("admin", Some(&instance.admin_password))
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .send()
            .await?;
        Ok(resp.json::<PlayersResponse>().await?)
    }

    /// Fetch server info (name, version, description).
    pub async fn get_server_info(&self, instance: &PalworldInstance) -> Result<ServerInfoResponse, Error> {
        let resp = self
            .http
            .get(Self::url(instance, "/v1/api/info"))
            .basic_auth("admin", Some(&instance.admin_password))
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .send()
            .await?;
        Ok(resp.json::<ServerInfoResponse>().await?)
    }

    /// Fetch live metrics (player count, FPS, uptime, etc.).
    pub async fn get_metrics(&self, instance: &PalworldInstance) -> Result<MetricsResponse, Error> {
        let resp = self
            .http
            .get(Self::url(instance, "/v1/api/metrics"))
            .basic_auth("admin", Some(&instance.admin_password))
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .send()
            .await?;
        Ok(resp.json::<MetricsResponse>().await?)
    }

    /// Fetch full server settings as a raw JSON value (sanitization is the caller's
    /// responsibility via `crate::utils::sanitize_sensitive_data`).
    pub async fn get_settings(&self, instance: &PalworldInstance) -> Result<serde_json::Value, Error> {
        let resp = self
            .http
            .get(Self::url(instance, "/v1/api/settings"))
            .basic_auth("admin", Some(&instance.admin_password))
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(format!("Server returned HTTP {}", resp.status()).into());
        }
        Ok(resp.json::<serde_json::Value>().await?)
    }

    // ─── Mutating operations ──────────────────────────────────────────────

    /// Broadcast a message to all connected players.
    pub async fn announce(&self, instance: &PalworldInstance, message: &str) -> Result<(), Error> {
        let resp = self
            .http
            .post(Self::url(instance, "/v1/api/announce"))
            .basic_auth("admin", Some(&instance.admin_password))
            .header("Content-Type", "application/json")
            .body(serde_json::json!({ "message": message }).to_string())
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(format!("HTTP {} — {}", status, body).into())
        }
    }

    /// Kick a player by user ID.
    pub async fn kick(&self, instance: &PalworldInstance, userid: &str, message: &str) -> Result<(), Error> {
        let resp = self
            .http
            .post(Self::url(instance, "/v1/api/kick"))
            .basic_auth("admin", Some(&instance.admin_password))
            .header("Content-Type", "application/json")
            .body(serde_json::json!({ "userid": userid, "message": message }).to_string())
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(format!("HTTP {} — {}", status, body).into())
        }
    }

    /// Ban a player by user ID.
    pub async fn ban(&self, instance: &PalworldInstance, userid: &str, message: &str) -> Result<(), Error> {
        let resp = self
            .http
            .post(Self::url(instance, "/v1/api/ban"))
            .basic_auth("admin", Some(&instance.admin_password))
            .header("Content-Type", "application/json")
            .body(serde_json::json!({ "userid": userid, "message": message }).to_string())
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(format!("HTTP {} — {}", status, body).into())
        }
    }

    /// Unban a previously banned player.
    pub async fn unban(&self, instance: &PalworldInstance, userid: &str) -> Result<(), Error> {
        let resp = self
            .http
            .post(Self::url(instance, "/v1/api/unban"))
            .basic_auth("admin", Some(&instance.admin_password))
            .header("Content-Type", "application/json")
            .body(serde_json::json!({ "userid": userid }).to_string())
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(format!("HTTP {} — {}", status, body).into())
        }
    }

    /// Save the current world state.
    pub async fn save(&self, instance: &PalworldInstance) -> Result<(), Error> {
        let resp = self
            .http
            .post(Self::url(instance, "/v1/api/save"))
            .basic_auth("admin", Some(&instance.admin_password))
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(format!("HTTP {} — {}", status, body).into())
        }
    }

    /// Initiate a graceful shutdown with an optional delay and message.
    pub async fn shutdown(
        &self,
        instance: &PalworldInstance,
        waittime: u64,
        message: &str,
    ) -> Result<StatusCode, Error> {
        let resp = self
            .http
            .post(Self::url(instance, "/v1/api/shutdown"))
            .basic_auth("admin", Some(&instance.admin_password))
            .body(serde_json::json!({ "waittime": waittime, "message": message }).to_string())
            .send()
            .await?;
        Ok(resp.status())
    }

    /// Immediately force-stop the server process.
    pub async fn force_stop(&self, instance: &PalworldInstance) -> Result<StatusCode, Error> {
        let resp = self
            .http
            .post(Self::url(instance, "/v1/api/stop"))
            .basic_auth("admin", Some(&instance.admin_password))
            .timeout(Duration::from_secs(3))
            .send()
            .await?;
        Ok(resp.status())
    }
}

impl Default for PalworldClient {
    fn default() -> Self {
        Self::new()
    }
}
