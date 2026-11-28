//! leptotes component tests for `Steps`.

use leptos::prelude::*;
use slp_core::{Coord, StepRun};

use super::{Steps, Transform};

fn t() -> Transform {
    Transform {
        px_ft: 10.0,
        pad: 0.0,
        yard_d: 20.0,
    }
}

fn run(elevation: f64) -> StepRun {
    StepRun {
        ax: 0.0,
        ay: 0.0,
        bx: 4.0,
        by: 0.0,
        elevation,
    }
}

#[test]
fn renders_a_footprint_and_treads_from_the_elevation() {
    // A 2 ft drop → 4 steps → 3 interior tread lines.
    let dom = leptotes::dom(
        move || view! { <Steps t=t() run=run(2.0) away_from=Coord::new(2.0, 5.0) /> },
    );

    let steps = dom.find("g.steps");

    assert_eq!(
        (steps.count("polygon"), steps.count("line.step-tread")),
        (1, 3),
        "{dom}"
    );
}

#[test]
fn one_step_run_has_no_interior_treads() {
    // A drop within one rise → a single step, so no interior tread lines.
    let dom = leptotes::dom(
        move || view! { <Steps t=t() run=run(0.4) away_from=Coord::new(2.0, 5.0) /> },
    );

    assert_eq!(dom.count("line.step-tread"), 0, "{dom}");
}

#[test]
fn footprint_extends_away_from_the_given_point_on_either_side() {
    // Same edge both times; away_from on opposite sides of it. px_ft=12 keeps
    // the far-edge screen coordinates whole numbers (depth * px_ft = 44).
    let t = Transform {
        px_ft: 12.0,
        pad: 0.0,
        yard_d: 20.0,
    };
    let vertices = |away_from: Coord| {
        let dom = leptotes::dom(move || view! { <Steps t=t run=run(2.0) away_from=away_from /> });
        dom.find("g.steps polygon")
            .attr("points")
            .unwrap_or_default()
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };

    let above = vertices(Coord::new(2.0, 5.0));
    let below = vertices(Coord::new(2.0, -5.0));

    // The edge sits at screen y 240; the far edge lands 44 px below it when
    // away_from is above the edge, and 44 px above it when it is below.
    assert_eq!(
        (
            above.iter().any(|p| p == "0,284"),
            below.iter().any(|p| p == "0,196")
        ),
        (true, true),
        "above: {above:?}, below: {below:?}"
    );
}
