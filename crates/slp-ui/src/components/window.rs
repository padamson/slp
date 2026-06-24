//! A window opening on a wall, drawn as a plan-view horizontal slice: the wall
//! is cut and the window reads as a thin frame box (the two wall faces + jambs)
//! with a glass pane line down the middle — the standard floor-plan symbol.
//! Positioned in SVG user-space pixels; the parent `Wall` converts feet→px.

use leptos::prelude::*;

/// Half the window frame's thickness, in px (how far the frame sits off the wall
/// centre line to each side).
const HALF_FRAME: f64 = 3.5;

#[component]
pub fn Window(x1: f64, y1: f64, x2: f64, y2: f64) -> impl IntoView {
    let (dx, dy) = (x2 - x1, y2 - y1);
    let len = dx.hypot(dy);
    // Unit normal to the wall, scaled to the half-frame thickness.
    let (nx, ny) = if len > 0.0 {
        (-dy / len * HALF_FRAME, dx / len * HALF_FRAME)
    } else {
        (0.0, HALF_FRAME)
    };
    // The two frame faces, offset to either side of the centre line.
    let (fx1, fy1, fx2, fy2) = (x1 + nx, y1 + ny, x2 + nx, y2 + ny);
    let (gx1, gy1, gx2, gy2) = (x1 - nx, y1 - ny, x2 - nx, y2 - ny);
    let gap = 2.0f64.mul_add(HALF_FRAME, 2.0);

    view! {
        <g class="window">
            // clear the wall under the opening (ground colour)
            <line x1=x1 y1=y1 x2=x2 y2=y2 stroke="#eef0e6" stroke-width=gap />
            // frame box: the two wall faces + a jamb closing each end
            <line class="window-frame" x1=fx1 y1=fy1 x2=fx2 y2=fy2 stroke="#5b6b7a" stroke-width="1" />
            <line class="window-frame" x1=gx1 y1=gy1 x2=gx2 y2=gy2 stroke="#5b6b7a" stroke-width="1" />
            <line class="window-frame" x1=fx1 y1=fy1 x2=gx1 y2=gy1 stroke="#5b6b7a" stroke-width="1" />
            <line class="window-frame" x1=fx2 y1=fy2 x2=gx2 y2=gy2 stroke="#5b6b7a" stroke-width="1" />
            // glass pane down the centre
            <line class="window-glass" x1=x1 y1=y1 x2=x2 y2=y2 stroke="#4a78a8" stroke-width="1.5" />
        </g>
    }
}
