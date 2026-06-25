//! The in-progress placement overlay: the nodes placed so far this gesture, the
//! chain between them, and a rubber-band edge to the previewed next node (which
//! follows the cursor). Tool-agnostic — the same overlay serves the house
//! outline and door/window spans. The committed plan is drawn by `House`; this
//! draws only what's being placed right now.

use leptos::prelude::*;
use slp_core::Coord;

use super::Transform;

#[component]
pub fn Placement(
    t: Transform,
    placed: Vec<Coord>,
    /// The previewed next node under the cursor (snapped), drawn as a ghost.
    #[prop(optional_no_strip)]
    preview: Option<Coord>,
) -> impl IntoView {
    (!placed.is_empty() || preview.is_some()).then(move || {
        // Solid chain through the nodes placed so far.
        let chain = (placed.len() >= 2).then(|| {
            let points = placed
                .iter()
                .map(|c| format!("{},{}", t.sx(c.x), t.sy(c.y)))
                .collect::<Vec<_>>()
                .join(" ");
            view! { <polyline points=points fill="none" stroke="#8a7f6a" stroke-width="2" /> }
        });

        let markers = placed
            .iter()
            .map(|c| {
                view! { <circle class="placement-node" cx=t.sx(c.x) cy=t.sy(c.y) r="3" fill="#8a7f6a" /> }
            })
            .collect::<Vec<_>>();

        // Rubber-band from the last placed node to the previewed node + a hollow
        // ghost marker where the next node would land.
        let last_px = placed.last().map(|c| (t.sx(c.x), t.sy(c.y)));
        let ghost = preview.map(|p| {
            let (px, py) = (t.sx(p.x), t.sy(p.y));
            let band = last_px.map(|(lx, ly)| {
                view! {
                    <line
                        class="placement-band"
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
                <g class="placement-preview">
                    {band}
                    <circle cx=px cy=py r="4" fill="none" stroke="#8a7f6a" stroke-dasharray="3 2" />
                </g>
            }
        });

        view! {
            <g class="placement">
                {chain}
                {markers}
                {ghost}
            </g>
        }
    })
}
