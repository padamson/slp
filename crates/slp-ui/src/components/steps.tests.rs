//! dokime component tests for `Steps`.

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

#[test]
fn renders_a_footprint_and_treads_from_the_elevation() {
    // A 2 ft drop → 4 steps → 3 interior tread lines.
    let run = StepRun {
        ax: 0.0,
        ay: 0.0,
        bx: 4.0,
        by: 0.0,
        elevation: 2.0,
    };
    let html =
        dokime::render(move || view! { <Steps t=t() run=run away_from=Coord::new(2.0, 5.0) /> });
    assert!(html.contains(r#"class="steps""#), "the run renders");
    assert!(html.contains("<polygon"), "the footprint polygon");
    assert_eq!(
        dokime::count(&html, r#"class="step-tread""#),
        3,
        "steps-1 interior treads"
    );
}

#[test]
fn one_step_run_has_no_interior_treads() {
    // A drop within one rise → a single step, so no interior tread lines.
    let run = StepRun {
        ax: 0.0,
        ay: 0.0,
        bx: 4.0,
        by: 0.0,
        elevation: 0.4,
    };
    let html =
        dokime::render(move || view! { <Steps t=t() run=run away_from=Coord::new(2.0, 5.0) /> });
    assert_eq!(dokime::count(&html, r#"class="step-tread""#), 0);
}
