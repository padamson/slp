//! A door opening on a wall: cuts a gap and draws a leaf + swing arc. Positioned
//! in SVG user-space pixels — the parent `Wall` converts feet→px. Composed by
//! `Wall`; never used standalone.

use leptos::prelude::*;

#[component]
pub fn Door(x1: f64, y1: f64, x2: f64, y2: f64) -> impl IntoView {
    // Hinge at (x1,y1); the leaf swings perpendicular to the gap (rotate 90°).
    let (dx, dy) = (x2 - x1, y2 - y1);
    let r = dx.hypot(dy);
    let (ox, oy) = (x1 - dy, y1 + dx);
    let arc = format!("M {x2} {y2} A {r} {r} 0 0 1 {ox} {oy}");
    view! {
        <g class="door">
            // cut the wall under the opening (ground colour)
            <line x1=x1 y1=y1 x2=x2 y2=y2 stroke="#eef0e6" stroke-width="5" />
            <line class="door-leaf" x1=x1 y1=y1 x2=ox y2=oy stroke="#b08968" stroke-width="2" />
            <path class="door-swing" d=arc fill="none" stroke="#b08968" stroke-width="1" />
        </g>
    }
}
