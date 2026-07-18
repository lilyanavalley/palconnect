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

/// Represents a community (Discord guild) that is using the bot.
///
/// In single-tenant mode there is exactly one `Tenant` synthesised from the local `Config.toml`.
/// In multi-tenant mode every guild that joins the bot gets its own `Tenant` record persisted in
/// the database.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Tenant {
    /// Opaque numeric identifier (auto-assigned in-process).
    pub id: u64,
    /// The Discord guild this tenant is bound to.  `None` while the single-tenant instance has not
    /// yet been invited to any guild.
    pub discord_guild_id: Option<u64>,
    /// Human-readable label for this tenant.
    pub name: String,
    /// Whether this tenant is active.  Disabled tenants are rejected at command entry.
    pub enabled: bool,
    /// Whether this tenant's guild may invite additional users to configure the bot (reserved for
    /// multi-tenant web-app flows; ignored in single-tenant mode).
    pub invite_allowed: bool,
    /// Optional Discord channel ID where the per-guild status message is pinned.
    pub status_channel_id: Option<u64>,
    /// Optional Discord message ID of the pinned status message inside `status_channel_id`.
    pub status_message_id: Option<u64>,
}

/// A single PalWorld dedicated-server REST API target owned by one `Tenant`.
///
/// Multiple tenants may each independently define a connection to the same physical game server;
/// ownership never crosses tenant boundaries.
#[derive(Debug, Clone)]
pub struct PalworldInstance {
    /// Opaque numeric identifier (auto-assigned in-process).
    pub id: u64,
    /// The owning tenant.
    pub tenant_id: u64,
    /// Human-readable label shown in Discord UX.
    pub display_name: String,
    /// Base URL of the PalWorld REST API, e.g. `http://10.0.0.1:8212`.
    pub api_url: String,
    /// Admin password for HTTP Basic Auth.  In single-tenant (self-hosted) mode this comes
    /// directly from the local config file and is never persisted elsewhere.  In multi-tenant mode
    /// it is stored AES-256-GCM encrypted and decrypted only at call time.
    pub admin_password: String,
    /// Whether this instance is active for polling and commands.
    pub enabled: bool,
    /// When a tenant has multiple instances, this one is the default target for commands that
    /// accept an implicit target.
    pub is_primary: bool,
}

/// Authorization policy associating a Discord role with a set of allowed commands for a specific
/// PalWorld instance (or all instances when `palworld_instance_id` is `None`).
///
/// Phase 1 note: `RolePolicy` rows exist in the data model and schema, but the `AuthzGuard`
/// enforcer is a no-op stub that always grants access.  Enforcement is deferred to a future
/// release.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RolePolicy {
    /// Opaque numeric identifier.
    pub id: u64,
    /// Owning tenant.
    pub tenant_id: u64,
    /// The PalWorld instance this policy applies to.  `None` means the policy applies to all
    /// instances owned by the tenant.
    pub palworld_instance_id: Option<u64>,
    /// Discord role ID whose members are granted the listed commands.
    pub discord_role_id: u64,
    /// List of slash-command names this role is permitted to invoke.
    pub allowed_commands: Vec<String>,
}
