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

use std::collections::HashMap;

use leptos::prelude::*;
use slp_core::{
    CatalogItem, Coord, FootprintShape, Object, Point, footprint_corners, within_a_single,
};

use super::Transform;
use crate::style::{
    DOUBLE_LINE_GAP_PX, DOUBLE_LINE_STROKE_W, FURNITURE_FILL, FURNITURE_STROKE, OVERFLOW_STROKE,
    SELECTED_FILL, SELECTED_STROKE, furniture_style,
};

/// Fallback footprint side (ft) when a catalog item carries no dimensions, so a
/// placed object is still visible and selectable.
const DEFAULT_FT: f64 = 1.0;

/// A resolved catalog footprint: its size in feet and whether it's a circle
/// (rendered as a `<circle>` of diameter `w_ft`) rather than a rectangle.
#[derive(Clone, Copy)]
struct Footprint {
    w_ft: f64,
    d_ft: f64,
    circle: bool,
}
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
    // rest. Each object's footprint is a width × depth rectangle centered at its
    // `(x, y)` and rotated `rot` degrees clockwise from north — the canvas draws
    // north up and SVG `rotate(+a)` turns clockwise in screen space, so the
    // schema's clockwise-from-north angle maps straight to `rotate(rot)`.
    let dims: HashMap<String, Footprint> = catalog
        .into_iter()
        .map(|c| {
            let circle = c.shape == FootprintShape::circle;
            let w_ft = c.width_ft.unwrap_or(DEFAULT_FT);
            // A circle uses its diameter (`width_ft`) for both axes, so its
            // bounding square — used by the fit-check and hit-test — is correct
            // regardless of `depth_ft`.
            let d_ft = if circle {
                w_ft
            } else {
                c.depth_ft.unwrap_or(DEFAULT_FT)
            };
            (c.id, Footprint { w_ft, d_ft, circle })
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
            let &fp = dims.get(&obj.catalog_ref)?;
            let rot = obj.rot.unwrap_or(0.0);
            let overflows = !surface_polys.is_empty()
                && !within_a_single(
                    &footprint_corners(obj.x, obj.y, fp.w_ft, fp.d_ft, rot),
                    &surface_polys,
                );
            let is_selected = selected == Some(i);
            Some(object_view(
                t,
                obj,
                i,
                fp,
                is_selected,
                overflows,
                on_handle_press,
                on_object_press,
            ))
        })
        .collect::<Vec<_>>();
    (!items.is_empty()).then(|| {
        view! { <g class="furnishings">{items}</g> }
    })
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
    on_handle_press: Option<Callback<()>>,
    on_object_press: Option<Callback<usize>>,
) -> impl IntoView {
    let Footprint { w_ft, d_ft, circle } = fp;
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
            {handle}
        </g>
    }
}
