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

/// Authorization guard for PalConnect commands.
///
/// Phase 1: always grants access.  The `RolePolicy` model and database schema are defined now so
/// that per-instance, per-command-group role enforcement can be added in a future release without
/// requiring changes to the command layer.
///
/// Future implementation will check whether the invoking user holds a Discord role that is granted
/// the requested command on the targeted `PalworldInstance`, using `RolePolicy` records from the
/// `TenantStore`.
pub struct AuthzGuard;

impl AuthzGuard {
    pub fn new() -> Self {
        Self
    }

    /// Check whether the current invocation is authorized.
    ///
    /// Returns `Ok(())` when access is granted, or an `Err` with a user-facing message when
    /// denied.  Phase 1 always returns `Ok(())`.
    pub async fn check(
        &self,
        _guild_id: Option<u64>,
        _user_id: u64,
        _command_name: &str,
        _instance_id: u64,
    ) -> Result<(), String> {
        // Phase 1 stub — enforcement deferred to a future release.
        Ok(())
    }
}

impl Default for AuthzGuard {
    fn default() -> Self {
        Self::new()
    }
}
