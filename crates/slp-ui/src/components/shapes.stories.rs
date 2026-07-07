//! theoria stories for `Shapes`. Compiled only under the `stories` feature.

use leptos::prelude::*;
use slp_core::{Coord, CurveEdge, Shape};

use super::{Shapes, Transform};
use theoria::Story;

fn t() -> Transform {
    Transform {
        px_ft: 12.0,
        pad: 40.0,
        yard_d: 30.0,
    }
}

fn rect(elevation: f64, bulges: Vec<f64>) -> Shape {
    Shape {
        corners: vec![
            Coord::new(8.0, 6.0),
            Coord::new(22.0, 6.0),
            Coord::new(22.0, 16.0),
            Coord::new(8.0, 16.0),
        ],
        elevation,
        bulges,
        curves: Vec::new(),
        material_ref: None,
        depth_in: None,
    }
}

pub fn stories() -> Vec<Story> {
    vec![
        Story::new("Structures/Shapes/A single area", || {
            view! {
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 460 400" width="460">
                    <Shapes t=t() shapes=vec![rect(0.0, Vec::new())] />
                </svg>
            }
        }),
        Story::new("Structures/Shapes/A raised area", || {
            view! {
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 460 400" width="460">
                    <Shapes t=t() shapes=vec![rect(0.5, Vec::new())] />
                </svg>
            }
        }),
        Story::new("Structures/Shapes/An area with a bowed (arc) edge", || {
            // The bottom edge bows outward into an arc.
            view! {
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 460 400" width="460">
                    <Shapes t=t() shapes=vec![rect(0.0, vec![0.0, 0.0, 0.6, 0.0])] />
                </svg>
            }
        }),
        Story::new(
            "Structures/Shapes/An area with a bezier (curved) edge",
            || {
                // Edge 2 (the top, node 2->3) is a cubic bezier bowing up.
                let mut shape = rect(0.0, Vec::new());
                shape.curves = vec![CurveEdge {
                    edge: 2,
                    control1: Box::new(Coord::new(18.0, 22.0)),
                    control2: Box::new(Coord::new(12.0, 22.0)),
                }];
                view! {
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 460 400" width="460">
                        <Shapes t=t() shapes=vec![shape] />
                    </svg>
                }
            },
        ),
        Story::new(
            "Structures/Shapes/A selected area shows node handles",
            || {
                view! {
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 460 400" width="460">
                        <Shapes t=t() shapes=vec![rect(0.0, Vec::new())] selected=Some(0) />
                    </svg>
                }
            },
        ),
        Story::new(
            "Structures/Shapes/Two adjacent nodes selected shows the insert popup",
            || {
                view! {
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 460 400" width="460">
                        <Shapes
                            t=t()
                            shapes=vec![rect(0.0, Vec::new())]
                            selected=Some(0)
                            selected_nodes=vec![0, 1]
                        />
                    </svg>
                }
            },
        ),
    ]
}
