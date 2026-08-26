use leptos::prelude::*;
use std::time::Duration;

const DISMISS_TICK_MS: u64 = 50;

/// Stickers display a fixed card to the app interface, typically used for persistent notes or messages.
///
/// `fixed position` is an optional parameter that allows you to specify the position of the sticker on the screen. If not provided, it defaults to "fixed bottom-0".
#[component(transparent)]
pub fn Sticker<'a>(children: Children, fixed_position: Option<&'a str>) -> impl IntoView {
    view! {
        <div class=format!("sticker {} text-xs", fixed_position.unwrap_or("fixed bottom-0"))>
            <div class="card max-w-lg bg-base-300 shadow-sm rounded-sm">
                <div class="card-body p-3">
                    {children()}
                </div>
            </div>
        </div>
    }
}

/// A sticker variant that can dismiss itself after a fixed number of seconds.
///
/// When `dismiss` is set, the sticker shows a countdown label and progress bar,
/// then hides itself when the timer expires.
#[component]
pub fn DismissibleSticker<'a>(
    children: ChildrenFn,
    fixed_position: Option<&'a str>,
    dismiss_sec: Option<u64>,
    dismiss_signal: Option<ReadSignal<bool>>
) -> impl IntoView {

    let sticker_class = format!(
        "sticker {} text-xs",
        fixed_position.unwrap_or("fixed bottom-0")
    );
    let (is_visible, set_is_visible) = signal(dismiss_sec != Some(0));
    let (is_hovering, set_is_hovering) = signal(false);
    let (remaining_millis, set_remaining_millis) = signal(dismiss_sec.unwrap_or(0) * 1000);

    if let Some(initial_seconds) = dismiss_sec {
        Effect::new(move |_| {

            if initial_seconds == 0 {
                set_is_visible.set(false);
                return;
            }

            let initial_millis = initial_seconds * 1000;
            let interval = set_interval_with_handle(
                move || {
                    if is_hovering.get_untracked() {
                        return;
                    }

                    set_remaining_millis.update(|millis| {
                        *millis = millis.saturating_sub(DISMISS_TICK_MS);
                        if *millis == 0 {
                            set_is_visible.set(false);
                        }
                    });
                },
                Duration::from_millis(DISMISS_TICK_MS),
            )
            .ok();

            set_remaining_millis.set(initial_millis);

            on_cleanup(move || {
                if let Some(interval) = interval {
                    interval.clear();
                }
            });

        });
    }

    view! {
        <Show when=move || is_visible.get() && dismiss_signal.is_some_and(|d| !d.get()) >
            // * Sticker body
            <div
                class=sticker_class.clone()
                on:mouseenter=move |_| set_is_hovering.set(true)
                on:mouseleave=move |_| set_is_hovering.set(false)
            >
                // * is really just a card with 'special sauce'
                <div class="card max-w-xl bg-base-300 card-border border-accent shadow shadow-accent overflow-hidden rounded-lg">
                    <div class="card-body gap-2 p-3">
                        { children() }
                    </div>
                    // * with an optional countdown
                    {move || {
                        dismiss_sec.map(|initial_seconds| {
                            let total_millis = (initial_seconds.max(1) * 1000).to_string();
                            view! {
                                <div class="flex flex-col items-center gap-3 text-[0.65rem] opacity-70">
                                    <progress
                                        class="progress progress-accent flex-1"
                                        max=total_millis.clone()
                                        value=move || remaining_millis.get().to_string()
                                    ></progress>
                                </div>
                            }
                        })
                    }}
                </div>
            </div>
        </Show>
    }
}
