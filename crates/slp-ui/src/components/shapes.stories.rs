//! theoria stories for `Shapes`. Compiled only under the `stories` feature.

use leptos::prelude::*;
use slp_core::{Coord, Shape};

use super::{Shapes, Transform};
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![
        Story::new("Structures/Shapes/A single area", || {
            let t = Transform {
                px_ft: 12.0,
                pad: 40.0,
                yard_d: 30.0,
            };
            let shape = Shape {
                corners: vec![
                    Coord::new(8.0, 6.0),
                    Coord::new(22.0, 6.0),
                    Coord::new(22.0, 16.0),
                    Coord::new(8.0, 16.0),
                ],
                elevation: 0.0,
            };
            view! {
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 460 400" width="460">
                    <Shapes t=t shapes=vec![shape] />
                </svg>
            }
        }),
        Story::new("Structures/Shapes/A raised area", || {
            let t = Transform {
                px_ft: 12.0,
                pad: 40.0,
                yard_d: 30.0,
            };
            let shape = Shape {
                corners: vec![
                    Coord::new(8.0, 6.0),
                    Coord::new(22.0, 6.0),
                    Coord::new(22.0, 16.0),
                    Coord::new(8.0, 16.0),
                ],
                elevation: 0.5,
            };
            view! {
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 460 400" width="460">
                    <Shapes t=t shapes=vec![shape] />
                </svg>
            }
        }),
    ]
}
