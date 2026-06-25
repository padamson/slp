//! theoria story for `Steps`. Compiled only under the `stories` feature.

use leptos::prelude::*;
use slp_core::{Coord, StepRun};

use super::{Steps, Transform};
use theoria::Story;

pub fn stories() -> Vec<Story> {
    vec![Story::new("Structures/Steps", || {
        let t = Transform {
            px_ft: 12.0,
            pad: 20.0,
            yard_d: 16.0,
        };
        // A 3 ft drop off a top edge; centroid above, so the run extends down.
        let run = StepRun {
            ax: 2.0,
            ay: 10.0,
            bx: 14.0,
            by: 10.0,
            elevation: 3.0,
        };
        view! {
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 220 220" width="220">
                <Steps t=t run=run away_from=Coord::new(8.0, 14.0) />
            </svg>
        }
    })]
}
