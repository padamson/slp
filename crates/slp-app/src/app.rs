//! Root component. Composes the page chrome with the `Yard` canvas. Toolbar and
//! side panel (estimate, selection) join the tree in later slices.

use leptos::prelude::*;

use slp_ui::Yard;

/// Pixels per foot in the SVG user space.
const PX_FT: f64 = 12.0;
/// Padding around the yard, in pixels.
const PAD: f64 = 40.0;
/// Yard size in feet (hard-coded for the skeleton; user-editable in Slice 1).
const YARD_W: f64 = 70.0;
const YARD_D: f64 = 30.0;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <header>
            <h1>"Simple Landscape Planner"</h1>
            <p class="sub">"Walking skeleton — your yard, drawn to scale."</p>
        </header>
        <Yard yard_w=YARD_W yard_d=YARD_D px_ft=PX_FT pad=PAD />
    }
}
