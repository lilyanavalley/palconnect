
use leptos::prelude::*;
use icondata::{ BsShieldLock, BsDatabase, BsPersonVcard, BsUiChecks, MdiMenu, RiLogoutCircleSystemLine };
use leptos_icons::Icon;


const DRAWER_ITEMS: [DrawerItem; 4] = [
    DrawerItem {
        label: "Discord Guilds",
        route: "/guilds",
        icon: BsShieldLock
    },
    DrawerItem {
        label: "Palworld Servers",
        route: "/servers",
        icon: BsDatabase
    },
    DrawerItem {
        label: "Your Users",
        route: "/users",
        icon: BsPersonVcard
    },
    DrawerItem {
        label: "Permission Sets",
        route: "/permissions",
        icon: BsUiChecks
    }
];

struct DrawerItem {

    pub label: &'static str,
    pub route: &'static str,
    pub icon: icondata::Icon

}

#[component(transparent)]
pub fn Drawer(children: Children) -> impl IntoView {
    view! {

        <div class="drawer lg:drawer-open">
            <input id="my-drawer-3" type="checkbox" class="drawer-toggle" />
            <div class="drawer-content">
                <div class="flex flex-col items-center justify-center h-full fixed z-10">
                    <label for="my-drawer-3" class="drawer-button btn btn-ghost btn-primary bg-primary/15 backdrop-blur-sm rounded-tl-0 rounded-bl-0 rounded-tr-full rounded-br-full shadow-lg lg:hidden">
                        <Icon icon=MdiMenu/>
                    </label>
                </div>
                <div class="flex flex-col overflow-y-scroll gap-2">
                    {children()}
                </div>
            </div>
            <div class="drawer-side bg-base-300/25 h-full">
                
                // Virtual overlay to close the drawer when clicking outside of it.
                <label for="my-drawer-3" aria-label="close sidebar" class="drawer-overlay"></label>

                <ul class="menu bg-base-100/50 backdrop-blur-sm p-2 gap-2 h-full w-3/4 rounded-tr-xl rounded-br-xl overflow-y-scroll shadow-lg">

                    // * Shows a user avatar in the drawer.
                    <div class="flex justify-center items-center gap-2">
                        <div class="avatar">
                            <div class="mask mask-squircle h-18">
                                <img src="https://img.daisyui.com/images/profile/demo/distracted1@192.webp" />
                            </div>
                        </div>
                        <div class="text-lg font-bold">
                            <span>"UserName"</span>
                        </div>
                    </div>

                    // TODO: Link to DRAWER_ITEMS
                    <ForEnumerate
                        each=move || DRAWER_ITEMS
                        key=|item| item.route
                        children={move |index, item| {
                            view! {
                                <li>
                                    <button class="btn btn-primary btn-block rounded-full flex justify-center is-drawer-open:justify-stretch items-center is-drawer-close:tooltip is-drawer-close:tooltip-right tooltip-primary transition-all ease-in-out duration-300" data-tip={ item.label }>
                                        <div>
                                            <Icon icon=item.icon/>
                                        </div>
                                        <span class="is-drawer-close:hidden">{ item.label }</span>
                                        // {move || index.get()} ". Value: " {move || counter.count.get()}
                                    </button>
                                </li>
                            }
                        }}
                    />

                    <li class="justify-self-end">
                        <button class="btn btn-error btn-sm btn-soft flex items-center place-self-end is-drawer-close:tooltip is-drawer-close:tooltip-right tooltip-primary transition-all ease-in-out duration-300" data-tip="Logout">
                            <div>
                                <Icon icon=RiLogoutCircleSystemLine/>
                            </div>
                            <span class="is-drawer-close:hidden">"Logout"</span>
                        </button>
                    </li>
                
                </ul>

            </div>
        </div>

    }
}
