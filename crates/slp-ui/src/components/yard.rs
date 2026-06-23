//! The yard canvas: the `<svg>` stage that composes the ground, the foot grid,
//! the house outline, and the scale bar. Clicks on the stage are translated from
//! screen pixels to world feet and reported via `on_pick` (used to draw the
//! house). Shapes, the deck layer, and more interaction land in later slices.

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
    #[prop(optional)]
    house: Vec<Coord>,
    /// Called with the click position in feet when the stage is clicked.
    #[prop(optional)]
    on_pick: Option<Callback<Coord>>,
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

    let on_click = move |ev: leptos::ev::MouseEvent| {
        if let (Some(cb), Some(at)) = (on_pick, pick_feet(&ev, t, w_px)) {
            cb.run(at);
        }
    };

    view! {
        <svg
            id="yard"
            data-testid="yard"
            xmlns="http://www.w3.org/2000/svg"
            viewBox=view_box
            width="100%"
            on:click=on_click
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
            <House t=t corners=house />
            <ScaleBar t=t baseline_y=baseline_y />
        </svg>
    }
}

/// Convert a click on the SVG stage to world feet. The stage preserves the
/// viewBox aspect ratio (so screen px scale uniformly to user-space px); we map
/// the click offset into user space, then invert the [`Transform`] to feet.
/// Browser-only — returns `None` under ssr / in tests.
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
