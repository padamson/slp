//! The yard canvas: the `<svg>` stage that composes the ground, the foot grid,
//! the committed house, the in-progress placement overlay, and the scale bar.
//! Pointer interaction follows the polyline-tool pattern: moving the mouse
//! previews the next node (`on_hover`), a release commits it (`on_commit`), and
//! leaving the stage clears the preview (`on_leave`). Positions are translated
//! from screen pixels to world feet.

use leptos::prelude::*;
use slp_core::{CatalogItem, Coord, DeckLevel, Object, Opening, StepRun};

use super::{CanvasMetrics, Deck, Furnishings, Grid, House, Placement, ScaleBar, Transform};

/// Fixed strip (px) reserved below the grid for the scale bar, independent of
/// the grid padding so the grid can sit flush to the canvas box.
const SCALE_BAR_ROOM: f64 = 30.0;

#[component]
pub fn Yard(
    yard_w: f64,
    yard_d: f64,
    px_ft: f64,
    pad: f64,
    /// The committed house outline corners (empty = no house).
    #[prop(optional, into)]
    house: Signal<Vec<Coord>>,
    /// The committed deck levels (empty = no deck).
    #[prop(optional, into)]
    deck: Signal<Vec<DeckLevel>>,
    /// Step runs on the deck's edges.
    #[prop(optional, into)]
    steps: Signal<Vec<StepRun>>,
    /// Committed doors/windows on the house walls.
    #[prop(optional, into)]
    openings: Signal<Vec<Opening>>,
    /// Objects placed on the plan (furniture, …).
    #[prop(optional, into)]
    objects: Signal<Vec<Object>>,
    /// The plan catalog, used to resolve each object's footprint.
    #[prop(optional, into)]
    catalog: Signal<Vec<CatalogItem>>,
    /// The selected object's index (into `objects`), if any.
    #[prop(optional, into)]
    selected: Signal<Option<usize>>,
    /// Nodes placed so far in the current placement gesture.
    #[prop(optional, into)]
    placed: Signal<Vec<Coord>>,
    /// The previewed next node under the cursor (snapped).
    #[prop(optional, into)]
    preview: Signal<Option<Coord>>,
    /// Mouse moved over the stage — preview the next node at this point (feet).
    #[prop(optional)]
    on_hover: Option<Callback<Coord>>,
    /// Mouse released on the stage — commit a node at this point (feet).
    #[prop(optional)]
    on_commit: Option<Callback<Coord>>,
    /// Pointer left the stage — clear the preview.
    #[prop(optional)]
    on_leave: Option<Callback<()>>,
    /// Report the canvas's rendered geometry, measured on mount + window resize.
    #[prop(optional)]
    on_metrics: Option<Callback<CanvasMetrics>>,
    /// The selected object's rotation handle was pressed — start a rotate drag.
    #[prop(optional)]
    on_handle_press: Option<Callback<()>>,
    /// An object body was pressed (by index) — select it and start a move drag.
    #[prop(optional)]
    on_object_press: Option<Callback<usize>>,
) -> impl IntoView {
    let t = Transform { px_ft, pad, yard_d };
    let w_px = t.sx(yard_w) + pad;
    // A fixed strip below the grid holds the scale bar; with pad = 0 (the app)
    // the grid is otherwise flush to the canvas box, so its edges line up with
    // the surrounding page layout.
    let h_px = t.sy(0.0) + SCALE_BAR_ROOM;
    let view_box = format!("0 0 {w_px} {h_px}");

    let ground_x = t.sx(0.0);
    let ground_y = t.sy(yard_d);
    let ground_w = yard_w * px_ft;
    let ground_h = yard_d * px_ft;
    let baseline_y = h_px - 16.0;

    let hover = move |ev: leptos::ev::MouseEvent| {
        if let (Some(cb), Some(at)) = (on_hover, pick_feet(&ev, t, w_px)) {
            cb.run(at);
        }
    };
    let commit = move |ev: leptos::ev::MouseEvent| {
        if let (Some(cb), Some(at)) = (on_commit, pick_feet(&ev, t, w_px)) {
            cb.run(at);
        }
    };
    let leave = move |_ev: leptos::ev::MouseEvent| {
        if let Some(cb) = on_leave {
            cb.run(());
        }
    };

    // Measure the rendered canvas once it's laid out, and on every window resize,
    // reporting it upward so consumers position against one measured value.
    #[cfg(feature = "csr")]
    if let Some(cb) = on_metrics {
        let emit = move || {
            if let Some(m) = measure_canvas(w_px, px_ft) {
                cb.run(m);
            }
        };
        Effect::new(move |_| {
            emit();
            let handle =
                leptos::prelude::window_event_listener(leptos::ev::resize, move |_| emit());
            on_cleanup(move || handle.remove());
        });
    }
    #[cfg(not(feature = "csr"))]
    let _ = on_metrics;

    view! {
        <svg
            id="yard"
            data-testid="yard"
            xmlns="http://www.w3.org/2000/svg"
            viewBox=view_box
            width="100%"
            on:mousemove=hover
            on:mouseup=commit
            on:mouseleave=leave
        >
            <rect
                x=ground_x
                y=ground_y
                width=ground_w
                height=ground_h
                fill="#eef0e6"
                stroke="#cfd3c0"
            />
            <Grid t=t yard_w=yard_w yard_d=yard_d />
            // Reactive overlays: only these subtrees update as the plan / gesture
            // change, so the <svg> stays put during a pointer gesture.
            {move || view! { <Deck t=t levels=deck.get() steps=steps.get() /> }}
            {move || view! { <House t=t corners=house.get() openings=openings.get() /> }}
            {move || {
                // Deck levels are the surfaces furniture should sit within (paver
                // areas join them when that slice lands).
                let surfaces = deck.get().into_iter().map(|l| l.corners).collect::<Vec<_>>();
                view! {
                    <Furnishings
                        t=t
                        objects=objects.get()
                        catalog=catalog.get()
                        surfaces=surfaces
                        selected=selected.get()
                        on_handle_press=on_handle_press
                        on_object_press=on_object_press
                    />
                }
            }}
            {move || view! { <Placement t=t placed=placed.get() preview=preview.get() /> }}
            <ScaleBar t=t baseline_y=baseline_y />
        </svg>
    }
}

/// Convert a pointer position on the SVG stage to world feet. The stage
/// preserves the viewBox aspect ratio (so screen px scale uniformly to
/// user-space px); we map the offset into user space, then invert the
/// [`Transform`] to feet. Browser-only — returns `None` under ssr / in tests.
#[cfg(feature = "csr")]
fn pick_feet(ev: &leptos::ev::MouseEvent, t: Transform, w_px: f64) -> Option<Coord> {
    use wasm_bindgen::JsCast;

    let svg: web_sys::Element = ev.current_target()?.dyn_into().ok()?;
    let rect = svg.get_bounding_client_rect();
    if rect.width() <= 0.0 {
        return None;
    }
    let scale = w_px / rect.width();
    let ux = (f64::from(ev.client_x()) - rect.left()) * scale;
    let uy = (f64::from(ev.client_y()) - rect.top()) * scale;
    Some(Coord::new(t.wx(ux), t.wy(uy)))
}

#[cfg(not(feature = "csr"))]
fn pick_feet(_ev: &leptos::ev::MouseEvent, _t: Transform, _w_px: f64) -> Option<Coord> {
    None
}

/// Measure the rendered `#yard` SVG into [`CanvasMetrics`]. `w_px`/`px_ft` are the
/// viewBox width and viewBox px-per-foot, used to convert the measured pixel size
/// into a rendered scale. Browser-only.
#[cfg(feature = "csr")]
fn measure_canvas(w_px: f64, px_ft: f64) -> Option<CanvasMetrics> {
    let svg = web_sys::window()?.document()?.get_element_by_id("yard")?;
    let rect = svg.get_bounding_client_rect();
    if rect.width() <= 0.0 {
        return None;
    }
    let scale = rect.width() / w_px; // rendered px per viewBox px
    Some(CanvasMetrics {
        left: rect.left(),
        top: rect.top(),
        px_ft: px_ft * scale,
        strip_px: SCALE_BAR_ROOM * scale,
    })
}
