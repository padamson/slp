//! The house outline: the user-drawn footprint, rendered to scale inside the
//! `Yard` as a closed SVG polygon with a marker at each corner (the markers give
//! feedback while drawing and become drag handles in a later editing slice). A
//! `preview` corner — the node being positioned while the mouse is held — is
//! drawn as a ghost marker with a rubber-band edge from the last corner. Doors
//! and windows land on its walls in H1.2. Nothing here is hardcoded — the
//! corners come from the `Plan`.

use leptos::prelude::*;
use slp_core::Coord;

use super::Transform;

#[component]
pub fn House(
    t: Transform,
    corners: Vec<Coord>,
    /// The node currently being positioned (mouse held), drawn as a ghost.
    #[prop(optional_no_strip)]
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

        // The node being positioned: a rubber-band edge from the last corner
        // plus a ghost marker at the snapped cursor.
        let ghost = preview.map(|p| {
            let edge = corners.last().map(|last| {
                view! {
                    <line
                        class="house-preview-edge"
                        x1=t.sx(last.x)
                        y1=t.sy(last.y)
                        x2=t.sx(p.x)
                        y2=t.sy(p.y)
                        stroke="#8a7f6a"
                        stroke-width="1.5"
                        stroke-dasharray="4 3"
                    />
                }
            });
            view! {
                <g class="house-preview">
                    {edge}
                    <circle
                        cx=t.sx(p.x)
                        cy=t.sy(p.y)
                        r="4"
                        fill="none"
                        stroke="#8a7f6a"
                        stroke-dasharray="3 2"
                    />
                </g>
            }
        });

        view! {
            <g class="house">
                {outline}
                {markers}
                {ghost}
            </g>
        }
    })
}
