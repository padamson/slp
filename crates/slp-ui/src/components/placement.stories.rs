//! theoria story for `Placement` — the in-progress drawing overlay. A static
//! snapshot: three nodes placed, with the fourth previewed under the cursor and
//! a rubber-band edge to it.

use leptos::prelude::*;
use slp_core::Coord;

use super::{Footprint, Placement, Transform};
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![
        Story::new("Canvas/Placement/Drawing nodes (plain marker)", || {
            let t = Transform {
                px_ft: 12.0,
                pad: 20.0,
                yard_d: 20.0,
            };
            let placed = vec![
                Coord::new(2.0, 4.0),
                Coord::new(14.0, 4.0),
                Coord::new(14.0, 14.0),
            ];
            view! {
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 220 220" width="220">
                    <Placement t=t placed=placed preview=Some(Coord::new(2.0, 14.0)) />
                </svg>
            }
        }),
        Story::new("Canvas/Placement/Armed item footprint preview", || {
            let t = Transform {
                px_ft: 12.0,
                pad: 20.0,
                yard_d: 20.0,
            };
            let fp = Footprint {
                w_ft: 4.0,
                d_ft: 4.0,
                circle: true,
                clearance_ft: None,
                category: None,
                trunk_ft: None,
            };
            view! {
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 220 220" width="220">
                    <Placement
                        t=t
                        placed=Vec::new()
                        preview=Some(Coord::new(9.0, 9.0))
                        object_footprint=Some(fp)
                    />
                </svg>
            }
        }),
    ]
}
