use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TenantAdminState {
    pub guild_id: u64,
    pub tenant_id: u64,
    pub tenant_name: String,
    pub enabled: bool,
    pub invite_allowed: bool,
    #[serde(default)]
    pub instances: Vec<PalworldInstanceConfig>,
    #[serde(default)]
    pub role_policies: Vec<RolePolicyConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PalworldInstanceConfig {
    pub id: Option<u64>,
    pub display_name: String,
    pub api_url: String,
    pub admin_password: String,
    pub enabled: bool,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RolePolicyConfig {
    pub id: Option<u64>,
    pub palworld_instance_id: Option<u64>,
    pub discord_role_id: u64,
    #[serde(default)]
    pub allowed_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConnectionTestRequest {
    pub api_url: String,
    pub admin_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConnectionTestResponse {
    pub online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BridgeErrorResponse {
    pub message: String,
}
