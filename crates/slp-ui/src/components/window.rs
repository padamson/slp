//! A window opening on a wall: cuts a gap and draws the glass line. Positioned
//! in SVG user-space pixels — the parent `Wall` converts feet→px. Composed by
//! `Wall`; never used standalone.

use leptos::prelude::*;

#[component]
pub fn Window(x1: f64, y1: f64, x2: f64, y2: f64) -> impl IntoView {
    view! {
        <g class="window">
            // cut the wall under the opening (ground colour)
            <line x1=x1 y1=y1 x2=x2 y2=y2 stroke="#eef0e6" stroke-width="5" />
            <line class="window-glass" x1=x1 y1=y1 x2=x2 y2=y2 stroke="#4a78a8" stroke-width="2" />
        </g>
    }
}
