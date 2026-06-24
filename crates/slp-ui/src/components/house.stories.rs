//! theoria stories for `House`. Compiled only under the `stories` feature.
//! Two variants show the composition: a bare outline, and a house whose walls
//! compose doors & windows (House → Wall → Door/Window).

use leptos::prelude::*;
use slp_core::{Coord, Opening, OpeningKind};

use super::{House, Transform};
use theoria::Story;

fn t() -> Transform {
    Transform {
        px_ft: 12.0,
        pad: 40.0,
        yard_d: 30.0,
    }
}

pub fn stories() -> Vec<Story> {
    vec![
        Story::new("Structures/House/Outline", || {
            // An L-shaped footprint, drawn to scale inside an SVG stage.
            let corners = vec![
                Coord::new(10.0, 6.0),
                Coord::new(34.0, 6.0),
                Coord::new(34.0, 18.0),
                Coord::new(22.0, 18.0),
                Coord::new(22.0, 24.0),
                Coord::new(10.0, 24.0),
            ];
            view! {
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 520 400" width="520">
                    <House t=t() corners=corners />
                </svg>
            }
        }),
        Story::new("Structures/House/Doors & windows", || {
            // A rectangular house whose walls compose openings.
            // Opening::new(kind, offset, wall, width); walls are edges 0..4.
            let corners = vec![
                Coord::new(10.0, 6.0),
                Coord::new(34.0, 6.0),
                Coord::new(34.0, 24.0),
                Coord::new(10.0, 24.0),
            ];
            let openings = vec![
                Opening::new(OpeningKind::door, 9.0, 0, 3.0),
                Opening::new(OpeningKind::window, 4.0, 1, 5.0),
                Opening::new(OpeningKind::window, 8.0, 2, 4.0),
                Opening::new(OpeningKind::window, 5.0, 3, 4.0),
            ];
            view! {
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 520 400" width="520">
                    <House t=t() corners=corners openings=openings />
                </svg>
            }
        }),
    ]
}
