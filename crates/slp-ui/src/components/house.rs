//! The house outline: the user-drawn footprint, rendered to scale inside the
//! `Yard` as a closed SVG polygon with a marker at each corner (the markers give
//! feedback while drawing and become drag handles in a later editing slice).
//! Doors and windows land on its walls in H1.2. Nothing here is hardcoded — the
//! corners come from the `Plan`.

use leptos::prelude::*;
use slp_core::Coord;

use super::Transform;

#[component]
pub fn House(t: Transform, corners: Vec<Coord>) -> impl IntoView {
    // Nothing drawn yet → render nothing at all (no empty group/outline).
    (!corners.is_empty()).then(move || {
        let markers = corners
            .iter()
            .map(|c| {
                view! { <circle class="house-corner" cx=t.sx(c.x) cy=t.sy(c.y) r="3" fill="#8a7f6a" /> }
            })
            .collect::<Vec<_>>();

        // A closed outline needs at least an edge; one point is just a marker.
        let outline = (corners.len() >= 2).then(|| {
            let points = corners
                .iter()
                .map(|c| format!("{},{}", t.sx(c.x), t.sy(c.y)))
                .collect::<Vec<_>>()
                .join(" ");
            view! {
                <polygon
                    points=points
                    fill="#d8d2c4"
                    fill-opacity="0.6"
                    stroke="#8a7f6a"
                    stroke-width="2"
                />
            }
        });

        view! {
            <g class="house">
                {outline}
                {markers}
            </g>
        }
    })
}
