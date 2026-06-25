//! theoria story for `Deck`. Compiled only under the `stories` feature.

use leptos::prelude::*;
use slp_core::Coord;

use super::{Deck, Transform};
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![Story::new("Structures/Deck", || {
        let t = Transform {
            px_ft: 12.0,
            pad: 40.0,
            yard_d: 30.0,
        };
        // An L-shaped deck off the back of a house.
        let corners = vec![
            Coord::new(8.0, 6.0),
            Coord::new(28.0, 6.0),
            Coord::new(28.0, 16.0),
            Coord::new(18.0, 16.0),
            Coord::new(18.0, 22.0),
            Coord::new(8.0, 22.0),
        ];
        view! {
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 460 400" width="460">
                <Deck t=t corners=corners />
            </svg>
        }
    })]
}
