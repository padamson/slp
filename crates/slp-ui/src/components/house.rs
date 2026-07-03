//! The house outline: the user-drawn footprint, rendered to scale inside the
//! `Yard` as a closed SVG polygon (the walls) with a marker at each corner. The
//! house *composes* a `Wall` per edge, and each `Wall` composes its `Door`/
//! `Window` openings — so `house = outline + walls(+ doors + windows)`. This
//! draws the *committed* plan; the in-progress outline being drawn is the
//! `Placement` overlay. Nothing here is hardcoded — it all comes from the `Plan`.

use leptos::prelude::*;
use slp_core::{Coord, Opening};

use super::{Transform, Wall};
use crate::style::{HOUSE_FILL, HOUSE_FILL_OPACITY, HOUSE_STROKE};

#[component]
pub fn House(
    t: Transform,
    corners: Vec<Coord>,
    /// Doors and windows, each keyed to a wall (edge) by index.
    #[prop(optional)]
    openings: Vec<Opening>,
) -> impl IntoView {
    render_house(t, corners, openings)
}

// The body lives in a plain fn (not the `#[component]`) so the line count —
// dominated by expanded `view!` macros for a composition root — can be allowed.
#[allow(clippy::too_many_lines)]
fn render_house(t: Transform, corners: Vec<Coord>, openings: Vec<Opening>) -> impl IntoView {
    // Nothing to draw → render nothing (no empty group/outline).
    (!corners.is_empty()).then(move || {
        let markers = corners
            .iter()
            .map(|c| {
                view! { <circle class="house-corner" cx=t.sx(c.x) cy=t.sy(c.y) r="3" fill=HOUSE_STROKE /> }
            })
            .collect::<Vec<_>>();

        // The footprint fill (the walls themselves are drawn by the Wall
        // children, so this is fill-only — no stroke — to avoid double edges).
        let outline = (corners.len() >= 2).then(|| {
            let points = corners
                .iter()
                .map(|c| format!("{},{}", t.sx(c.x), t.sy(c.y)))
                .collect::<Vec<_>>()
                .join(" ");
            view! { <polygon points=points fill=HOUSE_FILL fill-opacity=HOUSE_FILL_OPACITY stroke="none" /> }
        });

        // One Wall per edge. While drawing it's an open chain; once there are
        // three corners it's a closed ring (the last edge wraps to corner 0).
        let walls = (corners.len() >= 2).then(|| {
            let n = corners.len();
            let count = if n >= 3 { n } else { n - 1 };
            (0..count)
                .map(|i| {
                    let start = corners[i].clone();
                    let end = corners[(i + 1) % n].clone();
                    let on_wall: Vec<Opening> = openings
                        .iter()
                        .filter(|o| usize::try_from(o.wall).is_ok_and(|w| w == i))
                        .cloned()
                        .collect();
                    view! { <Wall t=t start=start end=end openings=on_wall /> }
                })
                .collect::<Vec<_>>()
        });

        view! {
            <g class="house">
                {outline}
                {walls}
                {markers}
            </g>
        }
    })
}
