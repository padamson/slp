//! The deck/patio footprint: the user-drawn outline, rendered to scale as a
//! closed polygon with a marker at each corner. This draws the *committed* deck
//! from the `Plan`; the in-progress outline being drawn is the `Placement`
//! overlay. Stairs, railing, and multiple levels come in later slices.

use leptos::prelude::*;
use slp_core::Coord;

use super::Transform;

#[component]
pub fn Deck(t: Transform, corners: Vec<Coord>) -> impl IntoView {
    (corners.len() >= 2).then(move || {
        let points = corners
            .iter()
            .map(|c| format!("{},{}", t.sx(c.x), t.sy(c.y)))
            .collect::<Vec<_>>()
            .join(" ");
        let markers = corners
            .iter()
            .map(|c| {
                view! { <circle class="deck-corner" cx=t.sx(c.x) cy=t.sy(c.y) r="3" fill="#8a6f4f" /> }
            })
            .collect::<Vec<_>>();
        view! {
            <g class="deck">
                <polygon
                    points=points
                    fill="#c8a97e"
                    fill-opacity="0.55"
                    stroke="#8a6f4f"
                    stroke-width="2"
                />
                {markers}
            </g>
        }
    })
}
