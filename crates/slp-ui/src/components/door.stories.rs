//! theoria story for `Door`. Compiled only under the `stories` feature. Shows a
//! door composed onto a real `Wall` (Door → Wall) — no hand-drawn scaffolding.

use leptos::prelude::*;
use slp_core::{Coord, Opening, OpeningKind};

use super::{Transform, Wall};
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![Story::new("Structures/Door", || {
        let t = Transform {
            px_ft: 12.0,
            pad: 20.0,
            yard_d: 10.0,
        };
        let openings = vec![Opening::new(OpeningKind::door, 4.0, 0, 3.0)];
        view! {
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 220 120" width="220">
                <Wall t=t start=Coord::new(1.0, 5.0) end=Coord::new(16.0, 5.0) openings=openings />
            </svg>
        }
    })]
}
