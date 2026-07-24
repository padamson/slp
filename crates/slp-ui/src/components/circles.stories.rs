//! theoria stories for `Circles`. Compiled only under the `stories` feature.

use leptos::prelude::*;
use slp_core::{Circle, Coord};

use super::{Circles, Transform};
use theoria::Story;

fn t() -> Transform {
    Transform {
        px_ft: 12.0,
        pad: 40.0,
        yard_d: 30.0,
    }
}

fn round(elevation: f64) -> Circle {
    Circle {
        center: Box::new(Coord::new(18.0, 12.0)),
        elevation,
        radius_ft: 6.0,
        material_ref: None,
        depth_in: None,
        pattern: None,
        courses: Vec::new(),
        borders: Vec::new(),
    }
}

pub fn stories() -> Vec<Story> {
    vec![
        Story::new("Structures/Circles/A single round area", || {
            view! {
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 460 400" width="460">
                    <Circles t=t() circles=vec![round(0.0)] />
                </svg>
            }
        }),
        Story::new("Structures/Circles/A raised round area", || {
            view! {
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 460 400" width="460">
                    <Circles t=t() circles=vec![round(0.5)] />
                </svg>
            }
        }),
        Story::new(
            "Structures/Circles/A selected area shows a resize handle",
            || {
                view! {
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 460 400" width="460">
                        <Circles t=t() circles=vec![round(0.0)] selected=Some(0) />
                    </svg>
                }
            },
        ),
    ]
}
