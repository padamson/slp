//! Placed objects (furniture first): each `Object`'s footprint rendered to scale
//! and rotated. The catalog supplies the footprint dimensions (width × depth);
//! the object supplies position and rotation. This draws the *committed* objects
//! from the `Plan` — every object is shown regardless of cost `status` (existing
//! and virtual items appear on the plan; status only affects the take-off).
//!
//! An object whose `catalog_ref` resolves to no catalog item is skipped (there is
//! no footprint to draw) — the same exclusion the cost take-off makes.
//!
//! When `surfaces` (deck levels, paver areas) are supplied, an object whose
//! footprint is not fully inside a single surface — it overhangs an edge, sits
//! off every surface, or straddles two — is outlined in red, so it's easy to see
//! at a glance what doesn't fit.

use std::collections::HashMap;

use leptos::prelude::*;
use slp_core::{CatalogItem, Coord, Object, Point, footprint_corners, within_a_single};

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
    /// Surfaces a placed object should sit within (deck levels, paver areas). An
    /// object not fully inside a single one is highlighted. Empty = no check.
    #[prop(optional)]
    surfaces: Vec<Vec<Coord>>,
    /// The index (into `objects`) of the currently selected object, if any — it
    /// renders with a selection tint.
    #[prop(default = None)]
    selected: Option<usize>,
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
    // Surface polygons in world points, once. Empty → skip the fit check.
    let surface_polys: Vec<Vec<Point>> = surfaces
        .into_iter()
        .map(|poly| poly.into_iter().map(|c| Point::new(c.x, c.y)).collect())
        .collect();
    let items = objects
        .into_iter()
        .enumerate()
        .filter_map(|(i, obj)| {
            let &(w_ft, d_ft) = dims.get(&obj.catalog_ref)?;
            let rot = obj.rot.unwrap_or(0.0);
            let overflows = !surface_polys.is_empty()
                && !within_a_single(
                    &footprint_corners(obj.x, obj.y, w_ft, d_ft, rot),
                    &surface_polys,
                );
            let is_selected = selected == Some(i);
            // Selection tints the fill; overflow colors the outline — both can show
            // on the same object (a selected piece that also doesn't fit).
            let mut class = String::from("furniture-item");
            if is_selected {
                class.push_str(" furniture-item--selected");
            }
            if overflows {
                class.push_str(" furniture-item--overflows");
            }
            let fill = if is_selected { "#7ea9d4" } else { "#a8927a" };
            let (stroke, stroke_w) = if overflows {
                ("#d4351c", "2.5")
            } else if is_selected {
                ("#2b6cb0", "2")
            } else {
                ("#5a4a3a", "1.5")
            };
            let (w_px, d_px) = (w_ft * t.px_ft, d_ft * t.px_ft);
            let transform = format!("translate({},{}) rotate({})", t.sx(obj.x), t.sy(obj.y), rot);
            Some(view! {
                <g class=class transform=transform>
                    <rect
                        x=-w_px / 2.0
                        y=-d_px / 2.0
                        width=w_px
                        height=d_px
                        fill=fill
                        fill-opacity="0.7"
                        stroke=stroke
                        stroke-width=stroke_w
                    />
                </g>
            })
        })
        .collect::<Vec<_>>();
    (!items.is_empty()).then(|| {
        view! { <g class="furnishings">{items}</g> }
    })
}
