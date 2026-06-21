// `must_use_candidate` is a false positive for Leptos view-returning fns.
#![allow(clippy::must_use_candidate)]

use leptos::mount::mount_to_body;
use leptos::prelude::*;

mod app;
use app::App;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
}
