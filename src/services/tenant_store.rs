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

use std::sync::RwLock;

use crate::config::Config;
use crate::models::{PalworldInstance, RolePolicy, Tenant};

// ─── TenantStore trait ───────────────────────────────────────────────────────

/// Abstraction over the tenant data source.
///
/// Phase 1 provides an in-memory implementation (`InMemoryTenantStore`) built from the local
/// `Config.toml`.  A future database-backed implementation for multi-tenant mode will satisfy the
/// same trait without requiring changes to the service layer or commands.
pub trait TenantStore: Send + Sync {
    /// Resolve the `Tenant` for the given Discord guild ID.
    ///
    /// In single-tenant mode this always returns the single synthesized tenant regardless of the
    /// guild ID.  In multi-tenant mode it looks up the tenant by guild ID and returns `None` if
    /// the guild has no associated tenant yet.
    fn get_tenant_for_guild(&self, guild_id: Option<u64>) -> Option<Tenant>;

    /// Return all PalWorld instances owned by the given tenant, in priority order (primary first).
    fn get_instances_for_tenant(&self, tenant_id: u64) -> Vec<PalworldInstance>;

    /// Return all role policies defined for the given tenant.
    ///
    /// Phase 1 stub: not yet enforced, reserved for Phase 5 role-based authorization.
    #[allow(dead_code)]
    fn get_role_policies_for_tenant(&self, tenant_id: u64) -> Vec<RolePolicy>;

    /// Whether the bot is running in multi-tenant mode.
    fn is_multi_tenant(&self) -> bool;

    /// Whether new guild invites are accepted (operator-level gate).
    fn is_invite_allowed(&self) -> bool;

    /// In single-tenant mode, return the Discord guild ID the single tenant is currently bound to,
    /// if any.  Always returns `None` in multi-tenant mode.
    fn get_single_tenant_guild_id(&self) -> Option<u64>;

    /// In single-tenant mode, bind the single tenant to the given Discord guild.  This is called
    /// on the first `GuildCreate` event so subsequent guild joins can be detected and rejected.
    /// No-op in multi-tenant mode.
    fn bind_single_tenant_guild(&self, guild_id: u64);

    /// Return all tenants.  Used by the glance polling loop to build per-guild status messages.
    fn get_all_tenants(&self) -> Vec<Tenant>;

    /// Update the status channel and message IDs for a tenant.
    fn set_tenant_status_channel(
        &self,
        tenant_id: u64,
        channel_id: Option<u64>,
        message_id: Option<u64>,
    );
}

// ─── Convenience helpers ─────────────────────────────────────────────────────

/// Resolve the primary (or only) enabled `PalworldInstance` for a tenant.
///
/// Returns the instance marked `is_primary`, falling back to the first enabled instance.
/// Returns `None` if the tenant has no enabled instances.
pub fn resolve_primary_instance(store: &dyn TenantStore, tenant_id: u64) -> Option<PalworldInstance> {
    let instances = store.get_instances_for_tenant(tenant_id);
    let enabled: Vec<_> = instances.into_iter().filter(|i| i.enabled).collect();
    enabled
        .iter()
        .find(|i| i.is_primary)
        .cloned()
        .or_else(|| enabled.into_iter().next())
}

/// Resolve the active tenant and its primary instance in a single call.
///
/// Returns a descriptive `String` error suitable for surfacing directly in Discord replies.
pub fn resolve_tenant_and_instance(
    store: &dyn TenantStore,
    guild_id: Option<u64>,
) -> Result<(Tenant, PalworldInstance), String> {
    let tenant = store
        .get_tenant_for_guild(guild_id)
        .ok_or_else(|| "⚠️ This server has not been configured yet. Please contact your administrator.".to_string())?;

    if !tenant.enabled {
        return Err("⚠️ PalConnect is currently disabled for this server.".to_string());
    }

    let instance = resolve_primary_instance(store, tenant.id)
        .ok_or_else(|| "⚠️ No PalWorld server has been configured for this Discord server. Please contact your administrator.".to_string())?;

    Ok((tenant, instance))
}

/// Resolve the active tenant and **all** its enabled instances in a single call.
///
/// Unlike [`resolve_tenant_and_instance`] (which returns only the primary), this returns every
/// enabled instance for the tenant so that callers can present a multi-instance selection UI or
/// aggregate results across all servers.
///
/// Returns a descriptive `String` error suitable for surfacing directly in Discord replies.
pub fn resolve_tenant_and_all_instances(
    store: &dyn TenantStore,
    guild_id: Option<u64>,
) -> Result<(Tenant, Vec<PalworldInstance>), String> {
    let tenant = store
        .get_tenant_for_guild(guild_id)
        .ok_or_else(|| "⚠️ This server has not been configured yet. Please contact your administrator.".to_string())?;

    if !tenant.enabled {
        return Err("⚠️ PalConnect is currently disabled for this server.".to_string());
    }

    let instances: Vec<PalworldInstance> = store
        .get_instances_for_tenant(tenant.id)
        .into_iter()
        .filter(|i| i.enabled)
        .collect();

    if instances.is_empty() {
        return Err("⚠️ No PalWorld server has been configured for this Discord server. Please contact your administrator.".to_string());
    }

    Ok((tenant, instances))
}

// ─── InMemoryTenantStore ─────────────────────────────────────────────────────

/// In-memory `TenantStore` implementation used by single-tenant (self-hosted) mode.
///
/// Populated from the local `Config.toml` at startup; no persistence layer is required.
/// Thread-safe via `RwLock`.
pub struct InMemoryTenantStore {
    tenants: RwLock<Vec<Tenant>>,
    instances: RwLock<Vec<PalworldInstance>>,
    /// Role policies are defined in the schema now; enforcement is deferred to Phase 5.
    #[allow(dead_code)]
    role_policies: RwLock<Vec<RolePolicy>>,
    multi_tenant: bool,
    invite_allowed: bool,
}

impl InMemoryTenantStore {
    /// Build a store from the application `Config`.
    ///
    /// In single-tenant mode this synthesizes one `Tenant` and one `PalworldInstance` per entry in
    /// `config.palworld_servers` (or the legacy `palworld_api_url` / `palworld_admin_password`
    /// fields for backward compatibility).
    pub fn from_config(config: &Config) -> Self {
        let multi_tenant = config.multi_tenant();
        let invite_allowed = config.invite_allowed();

        let tenant = Tenant {
            id: 1,
            discord_guild_id: None, // bound on first GuildCreate event
            name: "Default".to_string(),
            enabled: true,
            invite_allowed,
            status_channel_id: None,
            status_message_id: None,
        };

        // Build instances from the [[palworld_servers]] array, or fall back to the legacy single-
        // server fields for backward compatibility.
        let raw_servers = config.effective_palworld_servers();
        let instances: Vec<PalworldInstance> = raw_servers
            .into_iter()
            .enumerate()
            .map(|(idx, s)| PalworldInstance {
                id: (idx + 1) as u64,
                tenant_id: 1,
                display_name: s.name,
                api_url: s.api_url,
                admin_password: s.admin_password,
                enabled: true,
                is_primary: idx == 0, // first server is primary
            })
            .collect();

        Self {
            tenants: RwLock::new(vec![tenant]),
            instances: RwLock::new(instances),
            role_policies: RwLock::new(vec![]),
            multi_tenant,
            invite_allowed,
        }
    }
}

impl TenantStore for InMemoryTenantStore {
    fn get_tenant_for_guild(&self, _guild_id: Option<u64>) -> Option<Tenant> {
        // In single-tenant mode the one tenant always matches any guild.
        // Multi-tenant DB lookup will be implemented in Phase 4.
        self.tenants.read().unwrap().first().cloned()
    }

    fn get_instances_for_tenant(&self, tenant_id: u64) -> Vec<PalworldInstance> {
        self.instances
            .read()
            .unwrap()
            .iter()
            .filter(|i| i.tenant_id == tenant_id)
            .cloned()
            .collect()
    }

    fn get_role_policies_for_tenant(&self, tenant_id: u64) -> Vec<RolePolicy> {
        self.role_policies
            .read()
            .unwrap()
            .iter()
            .filter(|p| p.tenant_id == tenant_id)
            .cloned()
            .collect()
    }

    fn is_multi_tenant(&self) -> bool {
        self.multi_tenant
    }

    fn is_invite_allowed(&self) -> bool {
        self.invite_allowed
    }

    fn get_single_tenant_guild_id(&self) -> Option<u64> {
        if self.multi_tenant {
            return None;
        }
        self.tenants
            .read()
            .unwrap()
            .first()
            .and_then(|t| t.discord_guild_id)
    }

    fn bind_single_tenant_guild(&self, guild_id: u64) {
        if self.multi_tenant {
            return;
        }
        let mut tenants = self.tenants.write().unwrap();
        if let Some(t) = tenants.first_mut() {
            t.discord_guild_id = Some(guild_id);
        }
    }

    fn get_all_tenants(&self) -> Vec<Tenant> {
        self.tenants.read().unwrap().clone()
    }

    fn set_tenant_status_channel(
        &self,
        tenant_id: u64,
        channel_id: Option<u64>,
        message_id: Option<u64>,
    ) {
        let mut tenants = self.tenants.write().unwrap();
        if let Some(t) = tenants.iter_mut().find(|t| t.id == tenant_id) {
            t.status_channel_id = channel_id;
            t.status_message_id = message_id;
        }
    }
}
