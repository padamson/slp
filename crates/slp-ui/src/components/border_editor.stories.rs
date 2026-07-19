//! theoria story for `BorderEditor`. Compiled only under the `stories` feature.

use leptos::prelude::*;
use slp_core::Border;

use super::BorderEditor;
use theoria::Story;

fn noop_us() -> Callback<usize> {
    Callback::new(|_| {})
}
fn noop_um() -> Callback<(usize, String)> {
    Callback::new(|_| {})
}
fn noop_uw() -> Callback<(usize, f64)> {
    Callback::new(|_| {})
}
fn noop() -> Callback<()> {
    Callback::new(|()| {})
}

pub fn stories() -> Vec<Story> {
    vec![Story::new("Panels/BorderEditor/A double-ring edge", || {
        view! {
            <BorderEditor
                borders=vec![
                    Border::new("cobble".to_string(), 0.5),
                    Border::new("edging-stone".to_string(), 0.25),
                ]
                material_options=vec![
                    ("paver".to_string(), "Field pavers".to_string()),
                    ("cobble".to_string(), "Border cobble".to_string()),
                    ("edging-stone".to_string(), "Edging stones".to_string()),
                ]
                on_material=noop_um()
                on_width=noop_uw()
                on_add=noop()
                on_remove=noop_us()
            />
        }
    })]
}
