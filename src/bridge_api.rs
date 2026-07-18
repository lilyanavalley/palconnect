use std::sync::Arc;

use actix_web::{
    get, post, put,
    web::{self, Data, Json, Path},
    HttpRequest, HttpResponse, Responder,
};
use log::warn;
use palconnect_bridge::{
    BridgeErrorResponse, ConnectionTestRequest, ConnectionTestResponse, TenantAdminState,
};

use crate::models::PalworldInstance;
use crate::services::{InMemoryTenantStore, PalworldClient, TenantStore};

#[derive(Clone)]
pub struct BridgeApiState {
    pub tenant_store: Arc<InMemoryTenantStore>,
    pub palworld_client: Arc<PalworldClient>,
    pub bridge_api_token: Option<String>,
}

fn require_bridge_token(req: &HttpRequest, state: &BridgeApiState) -> Result<(), HttpResponse> {
    let Some(expected_token) = state.bridge_api_token.as_deref() else {
        return Err(HttpResponse::ServiceUnavailable().json(BridgeErrorResponse {
            message: "bridge API token is not configured on the bot".to_string(),
        }));
    };

    let provided_token = req
        .headers()
        .get("x-palconnect-bridge-token")
        .and_then(|value| value.to_str().ok());

    if provided_token == Some(expected_token) {
        Ok(())
    } else {
        warn!("Rejected bridge API request with an invalid token");
        Err(HttpResponse::Unauthorized().json(BridgeErrorResponse {
            message: "invalid bridge token".to_string(),
        }))
    }
}

#[get("/api/v1/admin/guilds/{guild_id}")]
pub async fn get_guild_admin_state(
    req: HttpRequest,
    path: Path<u64>,
    state: Data<BridgeApiState>,
) -> impl Responder {
    if let Err(resp) = require_bridge_token(&req, &state) {
        return resp;
    }

    let guild_id = path.into_inner();
    let guild_name = format!("Guild {}", guild_id);
    HttpResponse::Ok().json(state.tenant_store.ensure_admin_state_for_guild(guild_id, &guild_name))
}

#[put("/api/v1/admin/guilds/{guild_id}")]
pub async fn put_guild_admin_state(
    req: HttpRequest,
    path: Path<u64>,
    state: Data<BridgeApiState>,
    payload: Json<TenantAdminState>,
) -> impl Responder {
    if let Err(resp) = require_bridge_token(&req, &state) {
        return resp;
    }

    let guild_id = path.into_inner();
    let mut payload = payload.into_inner();
    payload.guild_id = guild_id;

    match state
        .tenant_store
        .replace_admin_state_for_guild(guild_id, payload)
    {
        Ok(updated) => HttpResponse::Ok().json(updated),
        Err(message) => HttpResponse::BadRequest().json(BridgeErrorResponse { message }),
    }
}

#[post("/api/v1/admin/test-connection")]
pub async fn post_test_connection(
    req: HttpRequest,
    state: Data<BridgeApiState>,
    payload: Json<ConnectionTestRequest>,
) -> impl Responder {
    if let Err(resp) = require_bridge_token(&req, &state) {
        return resp;
    }

    let payload = payload.into_inner();
    let probe = PalworldInstance {
        id: 0,
        tenant_id: 0,
        display_name: "Connection Test".to_string(),
        api_url: payload.api_url,
        admin_password: payload.admin_password,
        enabled: true,
        is_primary: true,
    };

    HttpResponse::Ok().json(ConnectionTestResponse {
        online: state.palworld_client.check_online(&probe).await,
    })
}
