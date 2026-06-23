//! The yard canvas: the `<svg>` stage that composes the ground, the foot grid,
//! the house outline, and the scale bar. Pointer interaction is press-to-aim /
//! release-to-place: pressing starts positioning a node, moving the held mouse
//! adjusts it (reported via `on_preview`), and releasing drops it (reported via
//! `on_pick`). All positions are translated from screen pixels to world feet.
//! Shapes, the deck layer, and more interaction land in later slices.

use leptos::prelude::*;
use slp_core::Coord;

use super::{Grid, House, ScaleBar, Transform};

#[component]
pub fn Yard(
    yard_w: f64,
    yard_d: f64,
    px_ft: f64,
    pad: f64,
    /// The house outline corners, if the user has drawn one (empty = no house).
    /// Read reactively so the stage persists while only the overlay updates.
    #[prop(optional, into)]
    house: Signal<Vec<Coord>>,
    /// The node being positioned while the mouse is held (drawn as a ghost).
    #[prop(optional, into)]
    preview: Signal<Option<Coord>>,
    /// Called with the drop position in feet when the mouse is released.
    #[prop(optional)]
    on_pick: Option<Callback<Coord>>,
    /// Called with the live position in feet while the mouse is held (`None`
    /// when positioning ends without a drop).
    #[prop(optional)]
    on_preview: Option<Callback<Option<Coord>>>,
) -> impl IntoView {
    let t = Transform { px_ft, pad, yard_d };
    let w_px = t.sx(yard_w) + pad;
    let h_px = t.sy(0.0) + pad;
    let view_box = format!("0 0 {w_px} {h_px}");

    let ground_x = t.sx(0.0);
    let ground_y = t.sy(yard_d);
    let ground_w = yard_w * px_ft;
    let ground_h = yard_d * px_ft;
    let baseline_y = h_px - 16.0;

    // Whether the mouse button is currently held over the stage.
    let holding = RwSignal::new(false);

    let on_down = move |ev: leptos::ev::MouseEvent| {
        if on_pick.is_none() && on_preview.is_none() {
            return;
        }
        if let Some(at) = pick_feet(&ev, t, w_px) {
            holding.set(true);
            if let Some(p) = on_preview {
                p.run(Some(at));
            }
        }
    };
    let on_move = move |ev: leptos::ev::MouseEvent| {
        if holding.get_untracked()
            && let Some(at) = pick_feet(&ev, t, w_px)
            && let Some(p) = on_preview
        {
            p.run(Some(at));
        }
    };
    let on_up = move |ev: leptos::ev::MouseEvent| {
        if !holding.get_untracked() {
            return;
        }
        holding.set(false);
        if let (Some(cb), Some(at)) = (on_pick, pick_feet(&ev, t, w_px)) {
            cb.run(at);
        }
        if let Some(p) = on_preview {
            p.run(None);
        }
    };
    // Leaving the stage while held cancels the in-progress node (no drop).
    let on_leave = move |_ev: leptos::ev::MouseEvent| {
        if holding.get_untracked() {
            holding.set(false);
            if let Some(p) = on_preview {
                p.run(None);
            }
        }
    };

    view! {
        <svg
            id="yard"
            data-testid="yard"
            xmlns="http://www.w3.org/2000/svg"
            viewBox=view_box
            width="100%"
            on:mousedown=on_down
            on:mousemove=on_move
            on:mouseup=on_up
            on:mouseleave=on_leave
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
            // Reactive overlay: only the house subtree updates as the outline /
            // preview change, so the <svg> stays put during a mouse gesture.
            {move || view! { <House t=t corners=house.get() preview=preview.get() /> }}
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
