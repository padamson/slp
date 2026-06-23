//! theoria story for `Wall`. Compiled only under the `stories` feature.

use leptos::prelude::*;
use slp_core::{Coord, Opening, OpeningKind};

use super::{Transform, Wall};
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![Story::new("Wall", || {
        let t = Transform {
            px_ft: 12.0,
            pad: 20.0,
            yard_d: 10.0,
        };
        let start = Coord::new(1.0, 5.0);
        let end = Coord::new(20.0, 5.0);
        let openings = vec![
            Opening::new(OpeningKind::door, 3.0, 0, 3.0),
            Opening::new(OpeningKind::window, 11.0, 0, 4.0),
        ];
        view! {
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 280 120" width="280">
                // the wall edge, so the openings read in context
                <line
                    x1=t.sx(start.x)
                    y1=t.sy(start.y)
                    x2=t.sx(end.x)
                    y2=t.sy(end.y)
                    stroke="#8a7f6a"
                    stroke-width="2"
                />
                <Wall t=t start=start end=end openings=openings />
            </svg>
        }
    })]
}
