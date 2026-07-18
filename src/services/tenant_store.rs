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
use palconnect_bridge::TenantAdminState;

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

    /// Ensure a tenant exists for the guild and return its current administrative state.
    fn ensure_admin_state_for_guild(&self, guild_id: u64, guild_name: &str) -> TenantAdminState;

    /// Return the administrative state for a guild without mutating storage.
    fn get_admin_state_for_guild(&self, guild_id: u64) -> Option<TenantAdminState>;

    /// Replace the editable administrative state for a guild.
    fn replace_admin_state_for_guild(
        &self,
        guild_id: u64,
        state: TenantAdminState,
    ) -> Result<TenantAdminState, String>;
}

// ─── User-facing error message constants ─────────────────────────────────────

const ERR_NOT_CONFIGURED: &str =
    "⚠️ This server has not been configured yet. Please contact your administrator.";
const ERR_DISABLED: &str = "⚠️ PalConnect is currently disabled for this server.";
const ERR_NO_INSTANCE: &str =
    "⚠️ No PalWorld server has been configured for this Discord server. Please contact your administrator.";
const DEFAULT_NEW_TENANT_NAME: &str = "Unnamed Guild";

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
        .ok_or(ERR_NOT_CONFIGURED)?;

    if !tenant.enabled {
        return Err(ERR_DISABLED.to_string());
    }

    let instance = resolve_primary_instance(store, tenant.id)
        .ok_or(ERR_NO_INSTANCE)?;

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
        .ok_or(ERR_NOT_CONFIGURED)?;

    if !tenant.enabled {
        return Err(ERR_DISABLED.to_string());
    }

    let instances: Vec<PalworldInstance> = store
        .get_instances_for_tenant(tenant.id)
        .into_iter()
        .filter(|i| i.enabled)
        .collect();

    if instances.is_empty() {
        return Err(ERR_NO_INSTANCE.to_string());
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
    next_tenant_id: RwLock<u64>,
    next_instance_id: RwLock<u64>,
    next_role_policy_id: RwLock<u64>,
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
        let server_count = raw_servers.len() as u64;
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
            next_tenant_id: RwLock::new(2),
            next_instance_id: RwLock::new(server_count + 1),
            next_role_policy_id: RwLock::new(1),
        }
    }

    fn next_id(counter: &RwLock<u64>) -> u64 {
        let mut guard = counter.write().unwrap();
        let next = *guard;
        *guard += 1;
        next
    }

    fn build_admin_state(&self, tenant: &Tenant) -> TenantAdminState {
        let instances = self
            .instances
            .read()
            .unwrap()
            .iter()
            .filter(|instance| instance.tenant_id == tenant.id)
            .map(|instance| palconnect_bridge::PalworldInstanceConfig {
                id: Some(instance.id),
                display_name: instance.display_name.clone(),
                api_url: instance.api_url.clone(),
                admin_password: instance.admin_password.clone(),
                enabled: instance.enabled,
                is_primary: instance.is_primary,
            })
            .collect();

        let role_policies = self
            .role_policies
            .read()
            .unwrap()
            .iter()
            .filter(|policy| policy.tenant_id == tenant.id)
            .map(|policy| palconnect_bridge::RolePolicyConfig {
                id: Some(policy.id),
                palworld_instance_id: policy.palworld_instance_id,
                discord_role_id: policy.discord_role_id,
                allowed_commands: policy.allowed_commands.clone(),
            })
            .collect();

        TenantAdminState {
            guild_id: tenant.discord_guild_id.unwrap_or_default(),
            tenant_id: tenant.id,
            tenant_name: tenant.name.clone(),
            enabled: tenant.enabled,
            invite_allowed: tenant.invite_allowed,
            instances,
            role_policies,
        }
    }

    fn ensure_multi_tenant(&self, guild_id: u64, guild_name: &str) -> Tenant {
        if let Some(existing) = self
            .tenants
            .read()
            .unwrap()
            .iter()
            .find(|tenant| tenant.discord_guild_id == Some(guild_id))
            .cloned()
        {
            return existing;
        }

        let tenant = Tenant {
            id: Self::next_id(&self.next_tenant_id),
            discord_guild_id: Some(guild_id),
            name: guild_name.to_string(),
            enabled: true,
            invite_allowed: self.invite_allowed,
            status_channel_id: None,
            status_message_id: None,
        };
        self.tenants.write().unwrap().push(tenant.clone());
        tenant
    }

    fn validate_role_policy_instance_ids(
        role_policies: &[palconnect_bridge::RolePolicyConfig],
        known_instance_ids: &[u64],
    ) -> Result<(), String> {
        for policy in role_policies {
            if let Some(instance_id) = policy.palworld_instance_id {
                if !known_instance_ids.contains(&instance_id) {
                    return Err(format!(
                        "role policy references unknown PalWorld instance ID {}",
                        instance_id
                    ));
                }
            }
        }
        Ok(())
    }
}

impl TenantStore for InMemoryTenantStore {
    fn get_tenant_for_guild(&self, guild_id: Option<u64>) -> Option<Tenant> {
        let tenants = self.tenants.read().unwrap();
        if self.multi_tenant {
            let guild_id = guild_id?;
            tenants
                .iter()
                .find(|tenant| tenant.discord_guild_id == Some(guild_id))
                .cloned()
        } else {
            tenants.first().cloned()
        }
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

    fn ensure_admin_state_for_guild(&self, guild_id: u64, guild_name: &str) -> TenantAdminState {
        if self.multi_tenant {
            let tenant = self.ensure_multi_tenant(guild_id, guild_name);
            self.build_admin_state(&tenant)
        } else {
            if self.get_single_tenant_guild_id().is_none() {
                self.bind_single_tenant_guild(guild_id);
            }
            let tenant = self
                .tenants
                .read()
                .unwrap()
                .first()
                .cloned()
                .expect("single-tenant store must contain a tenant");
            self.build_admin_state(&tenant)
        }
    }

    fn get_admin_state_for_guild(&self, guild_id: u64) -> Option<TenantAdminState> {
        let tenant = if self.multi_tenant {
            self.tenants
                .read()
                .unwrap()
                .iter()
                .find(|tenant| tenant.discord_guild_id == Some(guild_id))
                .cloned()
        } else {
            self.tenants.read().unwrap().first().cloned()
        }?;
        Some(self.build_admin_state(&tenant))
    }

    fn replace_admin_state_for_guild(
        &self,
        guild_id: u64,
        state: TenantAdminState,
    ) -> Result<TenantAdminState, String> {
        let tenant = if self.multi_tenant {
            self.ensure_multi_tenant(
                guild_id,
                if state.tenant_name.trim().is_empty() {
                    DEFAULT_NEW_TENANT_NAME
                } else {
                    &state.tenant_name
                },
            )
        } else {
            if self.get_single_tenant_guild_id().is_none() {
                self.bind_single_tenant_guild(guild_id);
            }
            self.tenants
                .read()
                .unwrap()
                .first()
                .cloned()
                .ok_or_else(|| "single-tenant store is missing its default tenant".to_string())?
        };

        let mut normalized_instances = state.instances;
        if !normalized_instances.is_empty() {
            let primary_count = normalized_instances
                .iter()
                .filter(|instance| instance.is_primary)
                .count();
            if primary_count == 0 {
                if let Some(first) = normalized_instances.first_mut() {
                    first.is_primary = true;
                }
            } else if primary_count > 1 {
                let mut seen_primary = false;
                for instance in &mut normalized_instances {
                    if instance.is_primary {
                        if seen_primary {
                            instance.is_primary = false;
                        } else {
                            seen_primary = true;
                        }
                    }
                }
            }
        }

        let instances: Vec<PalworldInstance> = normalized_instances
            .into_iter()
            .map(|instance| PalworldInstance {
                id: instance
                    .id
                    .unwrap_or_else(|| Self::next_id(&self.next_instance_id)),
                tenant_id: tenant.id,
                display_name: instance.display_name,
                api_url: instance.api_url,
                admin_password: instance.admin_password,
                enabled: instance.enabled,
                is_primary: instance.is_primary,
            })
            .collect();

        let known_instance_ids: Vec<u64> = instances.iter().map(|instance| instance.id).collect();
        Self::validate_role_policy_instance_ids(&state.role_policies, &known_instance_ids)?;
        let role_policies: Vec<RolePolicy> = state
            .role_policies
            .into_iter()
            .map(|policy| {
                Ok(RolePolicy {
                    id: policy
                        .id
                        .unwrap_or_else(|| Self::next_id(&self.next_role_policy_id)),
                    tenant_id: tenant.id,
                    palworld_instance_id: policy.palworld_instance_id,
                    discord_role_id: policy.discord_role_id,
                    allowed_commands: policy.allowed_commands,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        {
            let mut tenants = self.tenants.write().unwrap();
            let current = tenants
                .iter_mut()
                .find(|existing| existing.id == tenant.id)
                .ok_or_else(|| format!("tenant {} not found", tenant.id))?;
            current.discord_guild_id = Some(guild_id);
            current.name = state.tenant_name;
            current.enabled = state.enabled;
            current.invite_allowed = state.invite_allowed;
        }

        {
            let mut stored_instances = self.instances.write().unwrap();
            stored_instances.retain(|instance| instance.tenant_id != tenant.id);
            stored_instances.extend(instances);
        }

        {
            let mut stored_policies = self.role_policies.write().unwrap();
            stored_policies.retain(|policy| policy.tenant_id != tenant.id);
            stored_policies.extend(role_policies);
        }

        self.get_admin_state_for_guild(guild_id)
            .ok_or_else(|| format!("failed to rebuild tenant state for guild {}", guild_id))
    }
}
