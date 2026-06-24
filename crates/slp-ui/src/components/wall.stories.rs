//! theoria story for `Wall`. Compiled only under the `stories` feature. The wall
//! is self-contained (draws its own edge), so the story just composes it with a
//! door + a window — no hand-drawn scaffolding.

use leptos::prelude::*;
use slp_core::{Coord, Opening, OpeningKind};

use super::{Transform, Wall};
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![Story::new("Structures/Wall", || {
        let t = Transform {
            px_ft: 12.0,
            pad: 20.0,
            yard_d: 10.0,
        };
        let openings = vec![
            Opening::new(OpeningKind::door, 3.0, 0, 3.0),
            Opening::new(OpeningKind::window, 11.0, 0, 4.0),
        ];
        view! {
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 280 120" width="280">
                <Wall t=t start=Coord::new(1.0, 5.0) end=Coord::new(20.0, 5.0) openings=openings />
            </svg>
        }
    })]
}
