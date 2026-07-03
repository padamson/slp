//! A run of steps on an edge: a footprint extending outward from its top edge,
//! divided into treads. The step count + run depth are computed from the
//! `elevation` drop (standard rise/tread) — no railings. Composed by `Deck` now
//! (and the `House` later, for steps out of a door); `away_from` is the structure
//! centroid, so the run extends *outward*.

use leptos::prelude::*;
use slp_core::{Coord, StepRun, step_outward, step_run};

use super::Transform;
use crate::style::DECK_STROKE;

#[component]
pub fn Steps(t: Transform, run: StepRun, away_from: Coord) -> impl IntoView {
    steps_view(t, run, away_from)
}

// Body in a plain fn: `run` is a value-passthrough prop (by-value is intended).
#[allow(clippy::needless_pass_by_value)]
fn steps_view(t: Transform, run: StepRun, away_from: Coord) -> impl IntoView {
    let StepRun {
        ax,
        ay,
        bx,
        by,
        elevation,
    } = run;
    let edge_a = Coord::new(ax, ay);
    let edge_b = Coord::new(bx, by);
    let (steps, depth) = step_run(elevation);
    let (ox, oy) = step_outward(&edge_a, &edge_b, &away_from);
    // Footprint: top edge out to the far edge.
    let far_a = Coord::new(ox.mul_add(depth, ax), oy.mul_add(depth, ay));
    let far_b = Coord::new(ox.mul_add(depth, bx), oy.mul_add(depth, by));
    let points = [&edge_a, &edge_b, &far_b, &far_a]
        .iter()
        .map(|c| format!("{},{}", t.sx(c.x), t.sy(c.y)))
        .collect::<Vec<_>>()
        .join(" ");
    // Interior tread lines, parallel to the top edge.
    let treads = (1..steps)
        .map(|i| {
            let reach = depth * f64::from(i) / f64::from(steps);
            let near = Coord::new(ox.mul_add(reach, ax), oy.mul_add(reach, ay));
            let far = Coord::new(ox.mul_add(reach, bx), oy.mul_add(reach, by));
            view! {
                <line
                    class="step-tread"
                    x1=t.sx(near.x)
                    y1=t.sy(near.y)
                    x2=t.sx(far.x)
                    y2=t.sy(far.y)
                    stroke=DECK_STROKE
                    stroke-width="1"
                />
            }
        })
        .collect::<Vec<_>>();
    view! {
        <g class="steps">
            <polygon
                points=points
                fill="#d8c3a5"
                fill-opacity="0.7"
                stroke=DECK_STROKE
                stroke-width="1.5"
            />
            {treads}
        </g>
    }
}
