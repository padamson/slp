//! theoria stories for the controls panel — theoria dogfooding its own component
//! as a fixture (one level; not a Gallery-in-Gallery). Compiled only under the
//! `stories` feature.

use leptos::prelude::*;

use super::Controls;
use crate::{ArgControl, Story, story};

/// A **knobs** demo: edit the controls and watch the stage update live.
///
/// It exercises the autodocs panel against theoria's own components:
///
/// - `on` toggles the flag shown on the stage.
/// - `count` and `label` echo straight through.
///
/// The `#[story]` macro wires each arg's signal to the view (so a control edit
/// re-renders the stage) and captures the source shown under "Show code".
#[story(name = "Knobs · demo", on = true, count = 2.0, label = "hi")]
fn knobs(on: bool, count: f64, label: String) -> impl IntoView {
    view! {
        <div data-testid="knobs-out">
            <p class="k-label">{label}</p>
            <p class="k-count">{count}</p>
            <p class="k-flag">{if on { "ON" } else { "OFF" }}</p>
        </div>
    }
}

pub fn stories() -> Vec<Story> {
    vec![
        knobs(),
        Story::new("Controls · panel", || {
            view! {
                <Controls args=vec![
                    ("active", ArgControl::Bool(RwSignal::new(true))),
                    ("width_ft", ArgControl::Num(RwSignal::new(12.0))),
                    ("label", ArgControl::Text(RwSignal::new("Patio".to_string()))),
                ] />
            }
        }),
    ]
}
