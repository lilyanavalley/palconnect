
use leptos::{ prelude::*, logging::* };
use icondata::{ BsCodeSquare, BsShareFill, BsCodeSlash, BsCoin, BsCCircle, BsBan, BsRouter, BsGithub, MdiBrain, RiOpenSourceLogosLine };
use leptos_icons::Icon;
use crate::components::*;


#[component]
pub fn Credits() -> impl IntoView {
    view! {
        // <div class="credits flex flex-col justify-center items-center gap-6 min-h-full w-full p-3">

            <div class="card bg-base-200 w-full md:w-3/4 shadow">
                <div class="card-body">
                    <h2 class="card-title">
                        <div class="m-2">
                            <Icon icon=BsCodeSquare height="2rem"/>
                        </div>
                        "Authors"
                    </h2>
                    <div>
                        <ul class="list bg-base-100 rounded-box shadow-md">

                            <li class="list-row">
                                <div><img class="size-10 rounded-box" src="https://cdn.pronouns.page/images/01K418Z31QHACPG7M9BM79PWQJ-avatar.png"/></div>
                                <div>
                                    <div class="m-1">"Lily Ana Valley"</div>
                                    <div class="badge badge-xs badge-soft badge-secondary">"Owner"</div>
                                </div>
                                <a class="btn btn-circle btn-accent" href="https://github.com/lilyanavalley">
                                    <Icon icon=BsGithub/>
                                </a>
                            </li>
                            
                        </ul>
                    </div>
                </div>
            </div>

            <div class="card bg-base-200 w-full md:w-3/4 shadow">
                <div class="card-body">
                    <h2 class="card-title">
                        <div class="m-2">
                            <Icon icon=RiOpenSourceLogosLine height="2rem"/>
                        </div>
                        "Open Source"
                    </h2>
                    <p>
                        <div>
                            "This app is licensed under the "
                            <a href="https://www.gnu.org/licenses/agpl-3.0.html" class="link link-secondary">
                                "AGPL version 3 (or later)"
                            </a>
                            ". This license:"
                        </div>
                        <ul class="mt-6 flex flex-col gap-2 text-xs">
                            <li class="flex items-center gap-3">
                                <span class="text-info"><Icon icon=BsRouter height="0.75rem"/></span>
                                <span>"States 'Network Use' is Distribution"</span>
                            </li>
                            <li class="flex items-center gap-3">
                                <span class="text-success"><Icon icon=BsShareFill height="0.75rem"/></span>
                                <span>"Allows Sharing & Distributing"</span>
                            </li>
                            <li class="flex items-center gap-3">
                                <span class="text-success"><Icon icon=BsCodeSlash height="1rem"/></span>
                                <span>"Allows Modifing Source Code"</span>
                            </li>
                            <li class="flex items-center gap-3">
                                <span class="text-success"><Icon icon=BsCoin height="1rem"/></span>
                                <span>"Allows Use Privately & Commercially"</span>
                            </li>
                            <li class="flex items-center gap-3">
                                <span class="text-warning"><Icon icon=BsCCircle height="0.75rem"/></span>
                                <span>"Requires Including Copyright Notice"</span>
                            </li>
                            <li class="flex items-center gap-3">
                                <span class="text-warning"><Icon icon=BsCCircle height="0.75rem"/></span>
                                <span>"Requires Including AGPL Notice"</span>
                            </li>
                            <li class="flex items-center gap-3">
                                <span class="text-warning"><Icon icon=BsCCircle height="0.75rem"/></span>
                                <span>"Requires Stating Changes"</span>
                            </li>
                            <li class="flex items-center gap-3">
                                <span class="text-warning"><Icon icon=BsCCircle height="0.75rem"/></span>
                                <span>"Requires Disclosing Source"</span>
                            </li>
                            <li class="flex items-center gap-3">
                                <span class="text-warning"><Icon icon=BsCCircle height="0.75rem"/></span>
                                <span>"Requires Including Install Instructions"</span>
                            </li>
                            <li class="flex items-center gap-3">
                                <span class="text-error"><Icon icon=BsBan height="0.75rem"/></span>
                                <span>"Disallows Sublicensing"</span>
                            </li>
                            <li class="flex items-center gap-3">
                                <span class="text-error"><Icon icon=BsBan height="0.75rem"/></span>
                                <span>"Disallows Holding Authors Liable"</span>
                            </li>
                            <li class="flex items-center gap-3">
                                <span class="text-error"><Icon icon=BsBan height="0.75rem"/></span>
                                <span>"Comes With NO WARRANTY"</span>
                            </li>
                        </ul>
                    </p>
                    <div class="card-actions justify-end">
                        <button class="btn btn-secondary shadow shadow-secondary/50">"View Source"</button>
                    </div>
                </div>
            </div>

        // </div>
    }
}

