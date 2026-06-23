//! A wall — the edge from one house corner to the next — and the doors/windows
//! that sit on it. `House` renders one `Wall` per edge; the wall's *edge line* is
//! the house outline polygon, so `Wall` is responsible for composing the
//! openings on that edge (placing each `Door`/`Window` along it, in feet→px).

use leptos::prelude::*;
use slp_core::{Coord, Opening, OpeningKind, opening_segment};

use super::{Door, Transform, Window};

#[component]
pub fn Wall(t: Transform, start: Coord, end: Coord, openings: Vec<Opening>) -> impl IntoView {
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
    view! { <g class="wall">{marks}</g> }
}
