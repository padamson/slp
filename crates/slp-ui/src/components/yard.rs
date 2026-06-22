//! The yard canvas: the `<svg>` stage that composes the ground, the foot grid,
//! and the scale bar. Shapes, the deck layer, and interaction land in later
//! slices as additional children of this stage.

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

    view! {
        <svg
            id="yard"
            data-testid="yard"
            xmlns="http://www.w3.org/2000/svg"
            viewBox=view_box
            width="100%"
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
