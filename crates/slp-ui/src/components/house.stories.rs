//! theoria stories for `House`. Compiled only under the `stories` feature.

use leptos::prelude::*;
use slp_core::Coord;

use super::{House, Transform};
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![Story::new("House", || {
        let t = Transform {
            px_ft: 12.0,
            pad: 40.0,
            yard_d: 30.0,
        };
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
                <House t=t corners=corners />
            </svg>
        }
    })]
}
