//! theoria story for `Planner` — the composed, whole-app preview: edit a yard
//! dimension and watch the canvas reflow, in the gallery. Compiled only under
//! the `stories` feature.

use leptos::prelude::*;

use super::Planner;
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![Story::new("Planner", || view! { <Planner /> })]
}
