use leptos::{
    html::{Input, Textarea},
    prelude::*,
    task::spawn_local,
};
use leptos_meta::*;
use leptos_router::{
    components::{FlatRoutes, Route, Router},
    StaticSegment,
};
use palconnect_bridge::{
    BridgeErrorResponse, ConnectionTestRequest, ConnectionTestResponse, RolePolicyConfig,
    TenantAdminState,
};
use serde::de::DeserializeOwned;

fn apply_loaded_state(
    set_tenant_id: WriteSignal<u64>,
    set_guild_id: WriteSignal<String>,
    set_tenant_name: WriteSignal<String>,
    set_enabled: WriteSignal<bool>,
    set_invite_allowed: WriteSignal<bool>,
    set_instances_json: WriteSignal<String>,
    set_role_policies_json: WriteSignal<String>,
    state: TenantAdminState,
) {
    set_tenant_id.set(state.tenant_id);
    set_guild_id.set(state.guild_id.to_string());
    set_tenant_name.set(state.tenant_name);
    set_enabled.set(state.enabled);
    set_invite_allowed.set(state.invite_allowed);
    set_instances_json.set(
        serde_json::to_string_pretty(&state.instances).unwrap_or_else(|_| "[]".to_string()),
    );
    set_role_policies_json.set(
        serde_json::to_string_pretty(&state.role_policies).unwrap_or_else(|_| "[]".to_string()),
    );
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/palconnect-web.css" />
        <Title text="PalConnect Admin Console" />
        <Router>
            <FlatRoutes fallback=|| "Page not found.">
                <Route path=StaticSegment("") view=HomePage />
            </FlatRoutes>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let guild_id_ref = NodeRef::<Input>::new();
    let tenant_name_ref = NodeRef::<Input>::new();
    let instances_ref = NodeRef::<Textarea>::new();
    let role_policies_ref = NodeRef::<Textarea>::new();
    let test_url_ref = NodeRef::<Input>::new();
    let test_password_ref = NodeRef::<Input>::new();

    let (tenant_id, set_tenant_id) = signal(0_u64);
    let (guild_id, set_guild_id) = signal(String::new());
    let (tenant_name, set_tenant_name) = signal(String::new());
    let (enabled, set_enabled) = signal(true);
    let (invite_allowed, set_invite_allowed) = signal(true);
    let (instances_json, set_instances_json) = signal("[]".to_string());
    let (role_policies_json, set_role_policies_json) = signal("[]".to_string());
    let (status, set_status) = signal("Load a guild to begin.".to_string());
    let (connection_status, set_connection_status) = signal(String::new());

    view! {
        <main class="palconnect-admin">
            <section>
                <h1>"PalConnect multi-tenant admin console"</h1>
                <p>
                    "Load a Discord guild, edit its PalWorld server connections and role policies, and save the updated state back through the bot bridge API."
                </p>
            </section>

            <section>
                <label for="guild-id"><strong>"Discord guild ID"</strong></label>
                <div class="row">
                    <input id="guild-id" node_ref=guild_id_ref prop:value=move || guild_id.get() on:input=move |ev| set_guild_id.set(event_target_value(&ev)) />
                    <button on:click=move |_| {
                        let Some(input) = guild_id_ref.get() else {
                            set_status.set("Guild ID input is unavailable.".to_string());
                            return;
                        };
                        let raw_guild_id = input.value();
                        let Ok(parsed_guild_id) = raw_guild_id.parse::<u64>() else {
                            set_status.set("Enter a numeric Discord guild ID.".to_string());
                            return;
                        };
                        set_status.set("Loading guild state…".to_string());
                        let set_status = set_status;
                        let set_guild_id = set_guild_id;
                        let set_tenant_name = set_tenant_name;
                        let set_enabled = set_enabled;
                        let set_invite_allowed = set_invite_allowed;
                        let set_instances_json = set_instances_json;
                        let set_role_policies_json = set_role_policies_json;
                        spawn_local(async move {
                            match load_guild_state(parsed_guild_id).await {
                                Ok(state) => {
                                    apply_loaded_state(
                                        set_tenant_id,
                                        set_guild_id,
                                        set_tenant_name,
                                        set_enabled,
                                        set_invite_allowed,
                                        set_instances_json,
                                        set_role_policies_json,
                                        state,
                                    );
                                    set_status.set("Guild state loaded from the bot.".to_string());
                                }
                                Err(err) => {
                                    set_status.set(format!("Failed to load guild state: {}", err));
                                }
                            }
                        });
                    }>"Load guild"</button>
                </div>
            </section>

            <section>
                <h2>"Tenant settings"</h2>
                <label for="tenant-name">"Display name"</label>
                <input
                    id="tenant-name"
                    node_ref=tenant_name_ref
                    prop:value=move || tenant_name.get()
                    on:input=move |ev| set_tenant_name.set(event_target_value(&ev))
                />
                <div class="row">
                    <label>
                        <input
                            type="checkbox"
                            prop:checked=move || enabled.get()
                            on:change=move |ev| set_enabled.set(event_target_checked(&ev))
                        />
                        "Tenant enabled"
                    </label>
                    <label>
                        <input
                            type="checkbox"
                            prop:checked=move || invite_allowed.get()
                            on:change=move |ev| set_invite_allowed.set(event_target_checked(&ev))
                        />
                        "Allow guild invites"
                    </label>
                </div>
            </section>

            <section>
                <h2>"PalWorld server connections"</h2>
                <p>"Edit the JSON array directly. New items can omit an `id`; the bot will assign one."</p>
                <textarea
                    node_ref=instances_ref
                    prop:value=move || instances_json.get()
                    on:input=move |ev| set_instances_json.set(event_target_value(&ev))
                    rows="14"
                />
            </section>

            <section>
                <h2>"Discord role policies"</h2>
                <p>
                    "Policies can target a single PalWorld instance by ID or all instances by setting `palworld_instance_id` to null."
                </p>
                <textarea
                    node_ref=role_policies_ref
                    prop:value=move || role_policies_json.get()
                    on:input=move |ev| set_role_policies_json.set(event_target_value(&ev))
                    rows="14"
                />
            </section>

            <section>
                <div class="row">
                    <button on:click=move |_| {
                        let Ok(parsed_guild_id) = guild_id.get().parse::<u64>() else {
                            set_status.set("Load a numeric guild ID before saving.".to_string());
                            return;
                        };
                        let tenant_id = tenant_id.get();
                        let tenant_name = tenant_name.get();
                        let enabled = enabled.get();
                        let invite_allowed = invite_allowed.get();
                        let instances_json = instances_json.get();
                        let role_policies_json = role_policies_json.get();
                        set_status.set("Saving guild state through the bot bridge…".to_string());
                        let set_status = set_status;
                        let set_guild_id = set_guild_id;
                        let set_tenant_name = set_tenant_name;
                        let set_enabled = set_enabled;
                        let set_invite_allowed = set_invite_allowed;
                        let set_instances_json = set_instances_json;
                        let set_role_policies_json = set_role_policies_json;
                        spawn_local(async move {
                            match save_guild_state(
                                parsed_guild_id,
                                tenant_id,
                                tenant_name,
                                enabled,
                                invite_allowed,
                                instances_json,
                                role_policies_json,
                            )
                            .await
                            {
                                Ok(state) => {
                                    apply_loaded_state(
                                        set_tenant_id,
                                        set_guild_id,
                                        set_tenant_name,
                                        set_enabled,
                                        set_invite_allowed,
                                        set_instances_json,
                                        set_role_policies_json,
                                        state,
                                    );
                                    set_status.set("Guild state saved through the bot.".to_string());
                                }
                                Err(err) => {
                                    set_status.set(format!("Failed to save guild state: {}", err));
                                }
                            }
                        });
                    }>"Save guild state"</button>
                    <span>{move || status.get()}</span>
                </div>
            </section>

            <section>
                <h2>"Connection test"</h2>
                <div class="stack">
                    <input node_ref=test_url_ref placeholder="http://127.0.0.1:8212" />
                    <input node_ref=test_password_ref type="password" placeholder="Admin password" />
                    <button on:click=move |_| {
                        let Some(url_input) = test_url_ref.get() else {
                            set_connection_status.set("Connection URL input is unavailable.".to_string());
                            return;
                        };
                        let Some(password_input) = test_password_ref.get() else {
                            set_connection_status.set("Connection password input is unavailable.".to_string());
                            return;
                        };
                        let api_url = url_input.value();
                        let admin_password = password_input.value();
                        set_connection_status.set("Testing connection through the bot…".to_string());
                        let set_connection_status = set_connection_status;
                        spawn_local(async move {
                            match test_connection(api_url, admin_password).await {
                                Ok(response) if response.online => {
                                    set_connection_status.set("Bot bridge test succeeded; the PalWorld server is reachable.".to_string());
                                }
                                Ok(_) => {
                                    set_connection_status.set("Bot bridge test completed, but the PalWorld server did not respond successfully.".to_string());
                                }
                                Err(err) => {
                                    set_connection_status.set(format!("Failed to test connection: {}", err));
                                }
                            }
                        });
                    }>"Test PalWorld connection"</button>
                    <span>{move || connection_status.get()}</span>
                </div>
            </section>
        </main>
    }
}

#[server]
async fn load_guild_state(guild_id: u64) -> Result<TenantAdminState, ServerFnError> {
    bridge_get(&format!("/api/v1/admin/guilds/{guild_id}")).await
}

#[server]
async fn save_guild_state(
    guild_id: u64,
    tenant_id: u64,
    tenant_name: String,
    enabled: bool,
    invite_allowed: bool,
    instances_json: String,
    role_policies_json: String,
) -> Result<TenantAdminState, ServerFnError> {
    let instances = serde_json::from_str(&instances_json)
        .map_err(|err| ServerFnError::new(format!("invalid instances JSON: {err}")))?;
    let role_policies: Vec<RolePolicyConfig> = serde_json::from_str(&role_policies_json)
        .map_err(|err| ServerFnError::new(format!("invalid role policy JSON: {err}")))?;

    bridge_put(
        &format!("/api/v1/admin/guilds/{guild_id}"),
        &TenantAdminState {
            guild_id,
            tenant_id,
            tenant_name,
            enabled,
            invite_allowed,
            instances,
            role_policies,
        },
    )
    .await
}

#[server]
async fn test_connection(
    api_url: String,
    admin_password: String,
) -> Result<ConnectionTestResponse, ServerFnError> {
    bridge_post(
        "/api/v1/admin/test-connection",
        &ConnectionTestRequest {
            api_url,
            admin_password,
        },
    )
    .await
}

#[cfg(feature = "ssr")]
async fn bridge_get<T>(path: &str) -> Result<T, ServerFnError>
where
    T: DeserializeOwned,
{
    bridge_request::<(), T>(reqwest::Method::GET, path, None).await
}

#[cfg(feature = "ssr")]
async fn bridge_put<B, T>(path: &str, body: &B) -> Result<T, ServerFnError>
where
    B: serde::Serialize + ?Sized,
    T: DeserializeOwned,
{
    bridge_request(reqwest::Method::PUT, path, Some(body)).await
}

#[cfg(feature = "ssr")]
async fn bridge_post<B, T>(path: &str, body: &B) -> Result<T, ServerFnError>
where
    B: serde::Serialize + ?Sized,
    T: DeserializeOwned,
{
    bridge_request(reqwest::Method::POST, path, Some(body)).await
}

#[cfg(feature = "ssr")]
async fn bridge_request<B, T>(
    method: reqwest::Method,
    path: &str,
    body: Option<&B>,
) -> Result<T, ServerFnError>
where
    B: serde::Serialize + ?Sized,
    T: DeserializeOwned,
{
    let base_url = std::env::var("PALCONNECT_BOT_BRIDGE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
    let parsed_base_url = reqwest::Url::parse(&base_url)
        .map_err(|err| ServerFnError::new(format!("invalid PALCONNECT_BOT_BRIDGE_URL: {err}")))?;
    if !matches!(parsed_base_url.scheme(), "http" | "https") {
        return Err(ServerFnError::new(
            "PALCONNECT_BOT_BRIDGE_URL must use http or https",
        ));
    }
    let token = std::env::var("BRIDGE_API_TOKEN")
        .map_err(|_| ServerFnError::new("BRIDGE_API_TOKEN must be set for palconnect-web"))?;

    let client = reqwest::Client::new();
    let mut request = client
        .request(
            method,
            parsed_base_url
                .join(path.trim_start_matches('/'))
                .map_err(|err| ServerFnError::new(format!("invalid bridge path: {err}")))?,
        )
        .header("x-palconnect-bridge-token", token);

    if let Some(body) = body {
        request = request.json(body);
    }

    let response = request
        .send()
        .await
        .map_err(|err| ServerFnError::new(format!("bridge request failed: {err}")))?;
    let status = response.status();

    if status.is_success() {
        response
            .json::<T>()
            .await
            .map_err(|err| ServerFnError::new(format!("invalid bridge response: {err}")))
    } else if let Ok(error) = response.json::<BridgeErrorResponse>().await {
        Err(ServerFnError::new(error.message))
    } else {
        Err(ServerFnError::new(format!(
            "bridge request failed with HTTP {}",
            status
        )))
    }
}
