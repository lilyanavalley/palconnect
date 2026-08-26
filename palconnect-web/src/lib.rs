pub mod app;
pub mod components;
pub mod views;

#[cfg(feature = "hydrate")]
extern crate wasm_bindgen;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;

    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
