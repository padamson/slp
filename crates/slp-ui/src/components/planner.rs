//! The whole planner UI: header + yard controls + the to-scale yard canvas,
//! wired to shared `(width, depth)` signals. `slp-app` just mounts this — keeping
//! the root UI here (not in the binary) makes the entire app previewable in
//! theoria and testable with dokime. Grows with the toolbar, side panel, etc.

use leptos::prelude::*;

use super::{Yard, YardControls};

/// Pixels per foot in the SVG user space.
const PX_FT: f64 = 12.0;
/// Padding around the yard, in pixels.
const PAD: f64 = 40.0;
/// Default yard size in feet.
const DEFAULT_W: f64 = 70.0;
const DEFAULT_D: f64 = 30.0;

#[component]
pub fn Planner() -> impl IntoView {
    let (width, set_width) = signal(DEFAULT_W);
    let (depth, set_depth) = signal(DEFAULT_D);

    view! {
        <header>
            <h1>"Simple Landscape Planner"</h1>
            <p class="sub">"Set your yard size; the plan is drawn to scale."</p>
        </header>
        <YardControls width=width set_width=set_width depth=depth set_depth=set_depth />
        // Re-render the canvas whenever the dimensions change.
        {move || view! { <Yard yard_w=width.get() yard_d=depth.get() px_ft=PX_FT pad=PAD /> }}
    }
}
