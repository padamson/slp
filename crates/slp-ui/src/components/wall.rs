//! A wall — the edge from one house corner to the next — and the doors/windows
//! that sit on it. Self-contained: it draws its own edge line and composes a
//! `Door`/`Window` for each opening (placed along the edge, feet→px). `House`
//! renders one `Wall` per edge.

use leptos::prelude::*;
use slp_core::{Coord, Opening, OpeningKind, opening_segment};

use super::{Door, Transform, Window};
use crate::style::HOUSE_STROKE;

#[component]
pub fn Wall(t: Transform, start: Coord, end: Coord, openings: Vec<Opening>) -> impl IntoView {
    let (ex1, ey1, ex2, ey2) = (t.sx(start.x), t.sy(start.y), t.sx(end.x), t.sy(end.y));
    let marks = openings
        .into_iter()
        .map(move |o| {
            let (p, q) = opening_segment(&start, &end, o.offset, o.width);
            let (x1, y1, x2, y2) = (t.sx(p.x), t.sy(p.y), t.sx(q.x), t.sy(q.y));
            if o.kind == OpeningKind::door {
                view! { <Door x1=x1 y1=y1 x2=x2 y2=y2 /> }.into_any()
            } else {
                view! { <Window x1=x1 y1=y1 x2=x2 y2=y2 /> }.into_any()
            }
        })
        .collect::<Vec<_>>();
    view! {
        <g class="wall">
            <line class="wall-edge" x1=ex1 y1=ey1 x2=ex2 y2=ey2 stroke=HOUSE_STROKE stroke-width="2" />
            {marks}
        </g>
    }
}
