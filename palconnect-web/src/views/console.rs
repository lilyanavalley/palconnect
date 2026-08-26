
use leptos::{ prelude::*, logging::* };
use icondata::{ BsDatabase, BsPersonVcard, MdiServerNetwork, MdiGamepadSquare };
use leptos_icons::Icon;
use crate::components::*;


#[component]
pub fn Console() -> impl IntoView {

    let (affiliations_dismiss, set_affiliations_dismiss) = signal(false);
    let (status, set_status) = signal(GeneralStatusAreaContext::default());

    let status_server_count = move || status.read().server_count;
    let status_active_players = move || status.read().active_players;

    Effect::new(move || {
        log!("{} servers, {} players", status_server_count(), status_active_players());
    });

    view! {
        <div class="console flex flex-col min-h-full w-full p-3">

            // * Shows a disclaimer of no affiliation with PocketPair.
            <DismissibleSticker fixed_position=Some("fixed bottom-2 right-2") dismiss_sec=Some(15) dismiss_signal=Some(affiliations_dismiss)>
                <p class="text-sm text-center">"Not affiliated with PocketPair;"</p>
                <p class="text-sm">"PalConnect is an open-source community project"</p>
                <button
                    class="btn btn-accent btn-soft btn-sm"
                    on:click=move |_| {
                        set_affiliations_dismiss.set(true);
                    }
                >
                    "GOT IT!"
                </button>
            </DismissibleSticker>

            <GeneralStatusArea context=status/>

        </div>
    }
}

struct GeneralStatusAreaContext {
    server_count: usize,
    active_players: usize,
}

impl Default for GeneralStatusAreaContext {
    fn default() -> Self {
        GeneralStatusAreaContext { server_count: 0, active_players: 0 }
    }
}

// TODO: Consider turning into a component
#[component]
fn GeneralStatusArea(context: ReadSignal<GeneralStatusAreaContext>) -> impl IntoView {

    let server_count = move || context.read().server_count;
    let active_players = move || context.read().active_players;

    view! {

        <div class="card bg-base-200 w-full md:w-3/4 shadow">
            <div class="card-body">
                <div class="stats stats-vertical lg:stats-horizontal shadow">

                    <div class="stat">
                        <div class="stat-figure text-info text-3xl">
                            <Icon icon=MdiServerNetwork />
                        </div>
                        <div class="stat-title">"Servers"</div>
                        <div class="stat-value text-info">{server_count()}</div>
                        <div class="stat-desc">""</div>
                    </div>

                    <div class="stat">
                        <div class="stat-figure text-success text-3xl">
                            <Icon icon=MdiGamepadSquare />
                        </div>
                        <div class="stat-title">"Active Players"</div>
                        <div class="stat-value text-success">{active_players()}</div>
                        <div class="stat-desc">""</div>
                    </div>

                </div>
            </div>
        </div>

    }
}
