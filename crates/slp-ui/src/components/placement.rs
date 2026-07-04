//! The in-progress placement overlay: the nodes placed so far this gesture, the
//! chain between them, and a rubber-band edge to the previewed next node (which
//! follows the cursor). Tool-agnostic — the same overlay serves the house
//! outline and door/window spans. The committed plan is drawn by `House`; this
//! draws only what's being placed right now.
//!
//! When an object (not a house/deck/etc. node) is armed, the preview instead
//! shows a faint, shape-aware outline of the armed item's actual footprint —
//! reusing `Footprint`, the same resolution `Furnishings` draws committed
//! objects from — so you see *what* and *exactly where* it will land, not
//! just a center dot.

use leptos::prelude::*;
use slp_core::Coord;

use super::{Footprint, Transform};
use crate::style::{FURNITURE_FILL, FURNITURE_STROKE, PREVIEW_OPACITY};

#[component]
pub fn Placement(
    t: Transform,
    placed: Vec<Coord>,
    /// The previewed next node under the cursor (snapped), drawn as a ghost.
    #[prop(optional_no_strip)]
    preview: Option<Coord>,
    /// The armed catalog item's footprint, if the object tool is active — the
    /// preview draws this shape instead of the plain node marker.
    #[prop(default = None)]
    object_footprint: Option<Footprint>,
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

        // Rubber-band from the last placed node to the previewed node, plus
        // either the armed item's shape-aware footprint (object placement) or
        // the plain hollow-circle marker (house/deck/door/window/steps nodes).
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
            let marker = object_footprint.map_or_else(
                || {
                    view! {
                        <circle cx=px cy=py r="4" fill="none" stroke="#8a7f6a" stroke-dasharray="3 2" />
                    }
                    .into_any()
                },
                |fp| object_preview(t, px, py, fp),
            );
            view! {
                <g class="placement-preview">
                    {band}
                    {marker}
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

/// The armed item's footprint at the preview point: its real shape (rect or
/// circle), to scale, at a faint group opacity — "what and exactly where",
/// not a generic marker. Rotation isn't shown: a freshly placed object always
/// starts at 0°.
fn object_preview(t: Transform, px: f64, py: f64, fp: Footprint) -> AnyView {
    let (w_px, d_px) = (fp.w_ft * t.px_ft, fp.d_ft * t.px_ft);
    let shape = if fp.circle {
        view! {
            <circle cx=px cy=py r=w_px / 2.0 fill=FURNITURE_FILL stroke=FURNITURE_STROKE stroke-width="1.5" />
        }
        .into_any()
    } else {
        view! {
            <rect
                x=px - w_px / 2.0
                y=py - d_px / 2.0
                width=w_px
                height=d_px
                fill=FURNITURE_FILL
                stroke=FURNITURE_STROKE
                stroke-width="1.5"
            />
        }
        .into_any()
    };
    view! { <g class="placement-object-preview" opacity=PREVIEW_OPACITY>{shape}</g> }.into_any()
}
