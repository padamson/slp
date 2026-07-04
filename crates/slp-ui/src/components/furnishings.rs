//! Placed objects (furniture first): each `Object`'s footprint rendered to scale
//! and rotated. The catalog supplies the footprint dimensions (width × depth);
//! the object supplies position and rotation. This draws the *committed* objects
//! from the `Plan` — every object is shown regardless of cost `status`/
//! `is_virtual` (both only affect the take-off), but they still shape the
//! *look*, via two independent channels ([`crate::style::furniture_style`]):
//! **existing** items (already owned) render with a **double** outline (two
//! nested strokes) instead of **planned**'s single line, and **virtual** items
//! (a what-if ghost duplicate) render **dashed** and more transparent instead
//! of a real item's solid, full-color look — so status and realness both read
//! at a glance without opening the inspector.
//!
//! An object whose `catalog_ref` resolves to no catalog item is skipped (there is
//! no footprint to draw) — the same exclusion the cost take-off makes.
//!
//! When `surfaces` (deck levels, paver areas) are supplied, an object whose
//! footprint is not fully inside a single surface — it overhangs an edge, sits
//! off every surface, or straddles two — is outlined in red, so it's easy to see
//! at a glance what doesn't fit.
//!
//! A round item with a `clearance_ft` (a fire pit's keep-clear guideline) draws
//! a dashed **clearance ring** at `radius + clearance_ft`. The ring turns red
//! when another object's footprint or a `structure_outlines` edge (house/deck)
//! intrudes on it — always visible, not just when selected, since it's a
//! safety check.

use std::collections::HashMap;

use leptos::prelude::*;
use slp_core::{
    CatalogItem, Coord, Object, Point, circle_overlaps_circle, circle_overlaps_polygon,
    footprint_corners, within_a_single,
};

use super::{Footprint, Transform};
use crate::style::{
    CLEARANCE_STROKE, DOUBLE_LINE_GAP_PX, DOUBLE_LINE_STROKE_W, FURNITURE_FILL, FURNITURE_STROKE,
    OVERFLOW_STROKE, SELECTED_FILL, SELECTED_STROKE, furniture_style,
};

/// Rotation-handle geometry (viewBox px): gap from the footprint's north edge to
/// the handle, and the handle's radius.
const HANDLE_GAP_PX: f64 = 12.0;
const HANDLE_R: f64 = 5.0;

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
    /// Structure outlines (house, deck levels) whose *edges* count as a
    /// clearance-ring intrusion — a keep-clear zone shouldn't overlap a wall or
    /// deck edge either. Empty = no structures to check against.
    #[prop(optional)]
    structure_outlines: Vec<Vec<Coord>>,
    /// The index (into `objects`) of the currently selected object, if any — it
    /// renders with a selection tint and a rotation handle.
    #[prop(default = None)]
    selected: Option<usize>,
    /// The rotation handle was pressed — start a rotate gesture.
    #[prop(default = None)]
    on_handle_press: Option<Callback<()>>,
    /// An object's body was pressed (by `objects` index) — select it and start a
    /// move gesture.
    #[prop(default = None)]
    on_object_press: Option<Callback<usize>>,
) -> impl IntoView {
    // Resolve each catalog id to its footprint (consuming the catalog). One pass
    // instead of a linear scan per object; the object's `rot`/position handle the
    // rest. Each object's footprint is a width × depth rectangle (or a circle)
    // centered at its `(x, y)` and rotated `rot` degrees clockwise from north —
    // the canvas draws north up and SVG `rotate(+a)` turns clockwise in screen
    // space, so the schema's clockwise-from-north angle maps straight to
    // `rotate(rot)`.
    let dims: HashMap<String, Footprint> = catalog
        .into_iter()
        .map(|c| (c.id.clone(), Footprint::of(&c)))
        .collect();
    // Surface polygons in world points, once. Empty → skip the fit check.
    let surface_polys: Vec<Vec<Point>> = surfaces
        .into_iter()
        .map(|poly| poly.into_iter().map(|c| Point::new(c.x, c.y)).collect())
        .collect();
    let structure_polys: Vec<Vec<Point>> = structure_outlines
        .into_iter()
        .map(|poly| poly.into_iter().map(|c| Point::new(c.x, c.y)).collect())
        .collect();
    // Resolve every object's footprint up front (keeping the object alongside
    // it): the clearance-ring check needs random access to every *other*
    // object, not just the ones already visited in a single streaming pass.
    let resolved: Vec<(Object, Footprint)> = objects
        .into_iter()
        .filter_map(|obj| dims.get(&obj.catalog_ref).map(|&fp| (obj, fp)))
        .collect();
    let items = resolved
        .iter()
        .enumerate()
        .map(|(i, (obj, fp))| {
            let rot = obj.rot.unwrap_or(0.0);
            let overflows = !surface_polys.is_empty()
                && !within_a_single(
                    &footprint_corners(obj.x, obj.y, fp.w_ft, fp.d_ft, rot),
                    &surface_polys,
                );
            let is_selected = selected == Some(i);
            let intrudes = fp
                .clearance_ft
                .filter(|_| fp.circle)
                .is_some_and(|clearance| {
                    clearance_intrudes(
                        i,
                        obj,
                        fp.w_ft / 2.0 + clearance,
                        &resolved,
                        &structure_polys,
                    )
                });
            object_view(
                t,
                obj.clone(),
                i,
                *fp,
                is_selected,
                overflows,
                intrudes,
                on_handle_press,
                on_object_press,
            )
        })
        .collect::<Vec<_>>();
    (!items.is_empty()).then(|| {
        view! { <g class="furnishings">{items}</g> }
    })
}

/// Whether object `i`'s clearance disk (`center`, `radius`) overlaps any
/// *other* placed object's footprint, or any structure outline's edge.
fn clearance_intrudes(
    i: usize,
    obj: &Object,
    radius: f64,
    resolved: &[(Object, Footprint)],
    structure_polys: &[Vec<Point>],
) -> bool {
    let center = Point::new(obj.x, obj.y);
    let other_object_intrudes = resolved.iter().enumerate().any(|(j, (other, other_fp))| {
        if i == j {
            return false;
        }
        if other_fp.circle {
            circle_overlaps_circle(
                center,
                radius,
                Point::new(other.x, other.y),
                other_fp.w_ft / 2.0,
            )
        } else {
            let corners = footprint_corners(
                other.x,
                other.y,
                other_fp.w_ft,
                other_fp.d_ft,
                other.rot.unwrap_or(0.0),
            );
            circle_overlaps_polygon(center, radius, &corners)
        }
    });
    other_object_intrudes
        || structure_polys
            .iter()
            .any(|poly| circle_overlaps_polygon(center, radius, poly))
}

/// One object's footprint: its fill/outline (driven by selection, overflow,
/// and [`furniture_style`]), an inset second stroke when `existing` (a double
/// outline), and — when selected — a rotation handle.
///
/// `obj` is a by-value prop-like passthrough (matching `Steps`'s `run`):
/// Edition 2024's RPIT lifetime-capture rules mean a `&Object` here would tie
/// the returned `impl IntoView` to `obj`'s borrow, which the caller can't
/// satisfy (the object is a short-lived local in the iterator closure above).
#[allow(
    clippy::too_many_arguments,
    clippy::needless_pass_by_value,
    clippy::too_many_lines
)]
fn object_view(
    t: Transform,
    obj: Object,
    i: usize,
    fp: Footprint,
    is_selected: bool,
    overflows: bool,
    intrudes: bool,
    on_handle_press: Option<Callback<()>>,
    on_object_press: Option<Callback<usize>>,
) -> impl IntoView {
    let Footprint {
        w_ft,
        d_ft,
        circle,
        clearance_ft,
    } = fp;
    let rot = obj.rot.unwrap_or(0.0);
    // Selection tints the fill; overflow colors the outline — both can show on
    // the same object (a selected piece that also doesn't fit). Status/virtual
    // are an independent axis: they shape the outline count/dash/opacity, so
    // they still read through a selection tint or an overflow outline.
    let mut class = String::from("furniture-item");
    if is_selected {
        class.push_str(" furniture-item--selected");
    }
    if overflows {
        class.push_str(" furniture-item--overflows");
    }
    if intrudes {
        class.push_str(" furniture-item--intrudes");
    }
    let style = furniture_style(&obj.status, obj.is_virtual);
    class.push_str(style.class);
    let fill = if is_selected {
        SELECTED_FILL
    } else {
        FURNITURE_FILL
    };
    let (stroke, single_stroke_w) = if overflows {
        (OVERFLOW_STROKE, "2.5")
    } else if is_selected {
        (SELECTED_STROKE, "2")
    } else {
        (FURNITURE_STROKE, "1.5")
    };
    // A double (existing) outline uses two thin, closely-spaced lines so the
    // pair doesn't read as one heavy border.
    let stroke_w = if style.double {
        DOUBLE_LINE_STROKE_W
    } else {
        single_stroke_w
    };
    let (w_px, d_px) = (w_ft * t.px_ft, d_ft * t.px_ft);
    let r_px = w_px / 2.0; // circle radius (diameter = w_ft, so d_px == w_px)
    let transform = format!("translate({},{}) rotate({})", t.sx(obj.x), t.sy(obj.y), rot);
    // The footprint itself — a circle (fire pits, round tables) or a rectangle.
    let footprint = if circle {
        view! {
            <circle
                cx="0"
                cy="0"
                r=r_px
                fill=fill
                fill-opacity=style.fill_opacity
                stroke=stroke
                stroke-width=stroke_w
                stroke-dasharray=style.dash
            />
        }
        .into_any()
    } else {
        view! {
            <rect
                x=-w_px / 2.0
                y=-d_px / 2.0
                width=w_px
                height=d_px
                fill=fill
                fill-opacity=style.fill_opacity
                stroke=stroke
                stroke-width=stroke_w
                stroke-dasharray=style.dash
            />
        }
        .into_any()
    };
    // An `existing` item draws a second, inset stroke — a double outline reads
    // as "already owned" without needing a legend.
    let inner_outline = style.double.then(|| {
        let gap = DOUBLE_LINE_GAP_PX;
        if circle {
            view! {
                <circle
                    cx="0"
                    cy="0"
                    r=r_px - gap
                    fill="none"
                    stroke=stroke
                    stroke-width=stroke_w
                    stroke-dasharray=style.dash
                />
            }
            .into_any()
        } else {
            view! {
                <rect
                    x=-w_px / 2.0 + gap
                    y=-d_px / 2.0 + gap
                    width=w_px - 2.0 * gap
                    height=d_px - 2.0 * gap
                    fill="none"
                    stroke=stroke
                    stroke-width=stroke_w
                    stroke-dasharray=style.dash
                />
            }
            .into_any()
        }
    });
    // The rotation handle rides inside the rotated group (local north is -y),
    // so it turns with the object; pressing it starts a rotate drag.
    let handle = is_selected.then(|| {
        let stem = d_px / 2.0 + HANDLE_GAP_PX;
        view! {
            <g
                class="rotate-handle"
                data-testid="rotate-handle"
                on:mousedown=move |ev: leptos::ev::MouseEvent| {
                    ev.stop_propagation();
                    if let Some(cb) = on_handle_press {
                        cb.run(());
                    }
                }
            >
                <line x1="0" y1="0" x2="0" y2=-stem stroke=SELECTED_STROKE stroke-width="1" />
                <circle cx="0" cy=-stem r=HANDLE_R fill=SELECTED_STROKE />
            </g>
        }
    });
    // A round item's safety keep-clear zone (a fire pit's clearance guideline)
    // — always visible, not just when selected, since it's a safety check. It
    // turns red the instant something intrudes on it.
    let clearance_ring = clearance_ft.filter(|_| circle).map(|clearance_ft| {
        let ring_stroke = if intrudes {
            OVERFLOW_STROKE
        } else {
            CLEARANCE_STROKE
        };
        view! {
            <circle
                class="clearance-ring"
                data-testid="clearance-ring"
                cx="0"
                cy="0"
                r=r_px + clearance_ft * t.px_ft
                fill="none"
                stroke=ring_stroke
                stroke-width="1.5"
                stroke-dasharray="5,3"
            />
        }
    });
    view! {
        <g
            class=class
            transform=transform
            on:mousedown=move |_ev: leptos::ev::MouseEvent| {
                if let Some(cb) = on_object_press {
                    cb.run(i);
                }
            }
        >
            {footprint}
            {inner_outline}
            {clearance_ring}
            {handle}
        </g>
    }
}
