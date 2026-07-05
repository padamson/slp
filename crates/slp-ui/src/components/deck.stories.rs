//! theoria story for `Deck`. Compiled only under the `stories` feature.

use leptos::prelude::*;
use slp_core::{Coord, DeckLevel, StepRun};

use super::{Deck, Transform};
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![
        Story::new("Structures/Deck/Split level", || {
            let t = Transform {
                px_ft: 12.0,
                pad: 40.0,
                yard_d: 30.0,
            };
            // A split-level deck: a lower platform with a smaller upper one on top.
            let levels = vec![
                DeckLevel {
                    corners: vec![
                        Coord::new(8.0, 6.0),
                        Coord::new(30.0, 6.0),
                        Coord::new(30.0, 22.0),
                        Coord::new(8.0, 22.0),
                    ],
                    ..DeckLevel::new(1.0)
                },
                DeckLevel {
                    corners: vec![
                        Coord::new(18.0, 10.0),
                        Coord::new(30.0, 10.0),
                        Coord::new(30.0, 22.0),
                        Coord::new(18.0, 22.0),
                    ],
                    ..DeckLevel::new(3.0)
                },
            ];
            view! {
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 460 400" width="460">
                    <Deck t=t levels=levels />
                </svg>
            }
        }),
        Story::new("Structures/Deck/Single level with steps", || {
            let t = Transform {
                px_ft: 12.0,
                pad: 40.0,
                yard_d: 30.0,
            };
            let levels = vec![DeckLevel {
                corners: vec![
                    Coord::new(8.0, 6.0),
                    Coord::new(30.0, 6.0),
                    Coord::new(30.0, 22.0),
                    Coord::new(8.0, 22.0),
                ],
                ..DeckLevel::new(1.0)
            }];
            // A 2 ft drop off the front edge.
            let steps = vec![StepRun {
                ax: 8.0,
                ay: 6.0,
                bx: 30.0,
                by: 6.0,
                elevation: 1.0,
            }];
            view! {
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 460 400" width="460">
                    <Deck t=t levels=levels steps=steps />
                </svg>
            }
        }),
    ]
}
