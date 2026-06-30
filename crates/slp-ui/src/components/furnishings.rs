//! Placed objects (furniture first): each `Object`'s footprint rendered to scale
//! and rotated. The catalog supplies the footprint dimensions (width × depth);
//! the object supplies position and rotation. This draws the *committed* objects
//! from the `Plan` — every object is shown regardless of cost `status` (existing
//! and virtual items appear on the plan; status only affects the take-off).
//!
//! An object whose `catalog_ref` resolves to no catalog item is skipped (there is
//! no footprint to draw) — the same exclusion the cost take-off makes.

use std::collections::HashMap;

use leptos::prelude::*;
use slp_core::{CatalogItem, Object};

use super::Transform;

/// Fallback footprint side (ft) when a catalog item carries no dimensions, so a
/// placed object is still visible and selectable.
const DEFAULT_FT: f64 = 1.0;

#[component]
pub fn Furnishings(
    t: Transform,
    objects: Vec<Object>,
    /// The plan catalog, used to resolve each object's footprint dimensions.
    #[prop(optional)]
    catalog: Vec<CatalogItem>,
) -> impl IntoView {
    // Resolve each catalog id to its footprint (consuming the catalog). One pass
    // instead of a linear scan per object; the object's `rot`/position handle the
    // rest. Each object's footprint is a width × depth rectangle centered at its
    // `(x, y)` and rotated `rot` degrees clockwise from north — the canvas draws
    // north up and SVG `rotate(+a)` turns clockwise in screen space, so the
    // schema's clockwise-from-north angle maps straight to `rotate(rot)`.
    let dims: HashMap<String, (f64, f64)> = catalog
        .into_iter()
        .map(|c| {
            (
                c.id,
                (
                    c.width_ft.unwrap_or(DEFAULT_FT),
                    c.depth_ft.unwrap_or(DEFAULT_FT),
                ),
            )
        })
        .collect();
    let items = objects
        .into_iter()
        .filter_map(|obj| {
            let &(w_ft, d_ft) = dims.get(&obj.catalog_ref)?;
            let (w_px, d_px) = (w_ft * t.px_ft, d_ft * t.px_ft);
            let transform = format!(
                "translate({},{}) rotate({})",
                t.sx(obj.x),
                t.sy(obj.y),
                obj.rot.unwrap_or(0.0)
            );
            Some(view! {
                <g class="furniture-item" transform=transform>
                    <rect
                        x=-w_px / 2.0
                        y=-d_px / 2.0
                        width=w_px
                        height=d_px
                        fill="#a8927a"
                        fill-opacity="0.7"
                        stroke="#5a4a3a"
                        stroke-width="1.5"
                    />
                </g>
            })
        })
        .collect::<Vec<_>>();
    (!items.is_empty()).then(|| {
        view! { <g class="furnishings">{items}</g> }
    })
}
