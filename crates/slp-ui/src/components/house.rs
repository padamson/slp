//! The house outline: the user-drawn footprint, rendered to scale as a closed
//! SVG polygon inside the `Yard`. Doors and windows land on its walls in a later
//! slice. Nothing here is hardcoded — the corners come from the `Plan`.

use leptos::prelude::*;
use slp_core::Coord;

use super::Transform;

#[component]
pub fn House(t: Transform, corners: Vec<Coord>) -> impl IntoView {
    // Need at least an edge to draw; an empty/degenerate house draws nothing.
    (corners.len() >= 2).then(move || {
        let points = corners
            .into_iter()
            .map(|c| format!("{},{}", t.sx(c.x), t.sy(c.y)))
            .collect::<Vec<_>>()
            .join(" ");
        view! {
            <polygon
                class="house"
                points=points
                fill="#d8d2c4"
                fill-opacity="0.6"
                stroke="#8a7f6a"
                stroke-width="2"
            />
        }
    })
}
