//! A scale bar along the bottom of the canvas (default 10 ft).

use leptos::prelude::*;

use super::Transform;

#[component]
pub fn ScaleBar(
    t: Transform,
    #[prop(default = 10.0)] length_ft: f64,
    baseline_y: f64,
) -> impl IntoView {
    let x0 = t.sx(0.0);
    let x2 = x0 + length_ft * t.px_ft;
    let label_x = x0 + length_ft / 2.0 * t.px_ft;
    let label_y = baseline_y - 4.0;
    let label = format!("{length_ft:.0} ft");

    view! {
        <line x1=x0 y1=baseline_y x2=x2 y2=baseline_y stroke="#333" stroke-width="3" />
        <text x=label_x y=label_y text-anchor="middle" font-size="11" fill="#333">
            {label}
        </text>
    }
}
