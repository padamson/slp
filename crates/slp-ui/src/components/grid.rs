//! Foot grid over the yard: a faint line every foot, a darker one every 5 ft.

use leptos::prelude::*;

use super::Transform;

#[component]
pub fn Grid(t: Transform, yard_w: f64, yard_d: f64) -> impl IntoView {
    let x0 = t.sx(0.0);
    let xw = t.sx(yard_w);
    let y0 = t.sy(0.0);
    let yd = t.sy(yard_d);

    let verticals: Vec<_> = (0..=yard_w as i32)
        .map(|i| {
            let x = t.sx(f64::from(i));
            let stroke = if i % 5 == 0 { "#0000001c" } else { "#0000000d" };
            view! { <line x1=x y1=y0 x2=x y2=yd stroke=stroke stroke-width="1" /> }
        })
        .collect();

    let horizontals: Vec<_> = (0..=yard_d as i32)
        .map(|i| {
            let y = t.sy(f64::from(i));
            let stroke = if i % 5 == 0 { "#0000001c" } else { "#0000000d" };
            view! { <line x1=x0 y1=y x2=xw y2=y stroke=stroke stroke-width="1" /> }
        })
        .collect();

    view! {
        <g class="grid">
            {verticals}
            {horizontals}
        </g>
    }
}
