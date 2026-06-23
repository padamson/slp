//! The house outline: the user-drawn footprint, rendered to scale inside the
//! `Yard` as a closed SVG polygon (the walls) with a marker at each corner. The
//! house *composes* a `Wall` per edge, and each `Wall` composes its `Door`/
//! `Window` openings — so `house = outline + walls(+ doors + windows)`. A
//! `preview` corner — the node being positioned while the mouse is held — is
//! drawn as a ghost. Nothing here is hardcoded — it all comes from the `Plan`.

use leptos::prelude::*;
use slp_core::{Coord, Opening};

use super::{Transform, Wall};

#[component]
pub fn House(
    t: Transform,
    corners: Vec<Coord>,
    /// Doors and windows, each keyed to a wall (edge) by index.
    #[prop(optional)]
    openings: Vec<Opening>,
    /// The node currently being positioned (mouse held), drawn as a ghost.
    #[prop(optional_no_strip)]
    preview: Option<Coord>,
) -> impl IntoView {
    render_house(t, corners, openings, preview)
}

// The body lives in a plain fn (not the `#[component]`) so the line count —
// dominated by expanded `view!` macros for a composition root — can be allowed.
#[allow(clippy::too_many_lines)]
fn render_house(
    t: Transform,
    corners: Vec<Coord>,
    openings: Vec<Opening>,
    preview: Option<Coord>,
) -> impl IntoView {
    // Nothing to draw at all → render nothing (no empty group/outline).
    (!corners.is_empty() || preview.is_some()).then(move || {
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

        // One Wall per edge of the closed ring, each carrying its openings.
        let walls = (corners.len() >= 3).then(|| {
            let n = corners.len();
            (0..n)
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

        // The node being positioned: a ghost marker + rubber-band edge.
        let ghost = preview.map(|p| {
            let last_px = corners.last().map(|c| (t.sx(c.x), t.sy(c.y)));
            preview_ghost(last_px, (t.sx(p.x), t.sy(p.y)))
        });

        view! {
            <g class="house">
                {outline}
                {walls}
                {markers}
                {ghost}
            </g>
        }
    })
}

/// The ghost shown while positioning a node (coords in SVG px): a dashed
/// rubber-band edge from the last placed corner (if any) plus a hollow marker.
fn preview_ghost(last: Option<(f64, f64)>, (px, py): (f64, f64)) -> impl IntoView {
    let edge = last.map(|(lx, ly)| {
        view! {
            <line
                class="house-preview-edge"
                x1=lx
                y1=ly
                x2=px
                y2=py
                stroke="#8a7f6a"
                stroke-width="1.5"
                stroke-dasharray="4 3"
            />
        }
    });
    view! {
        <g class="house-preview">
            {edge}
            <circle cx=px cy=py r="4" fill="none" stroke="#8a7f6a" stroke-dasharray="3 2" />
        </g>
    }
}
