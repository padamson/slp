//! dokime component tests for `Deck` (multi-level footprints).

use leptos::prelude::*;
use slp_core::{Coord, DeckLevel, StepRun};

use super::{Deck, Transform};

fn t() -> Transform {
    Transform {
        px_ft: 10.0,
        pad: 0.0,
        yard_d: 20.0,
    }
}

fn square(elevation: f64) -> DeckLevel {
    DeckLevel {
        corners: vec![
            Coord::new(0.0, 0.0),
            Coord::new(4.0, 0.0),
            Coord::new(4.0, 3.0),
            Coord::new(0.0, 3.0),
        ],
        elevation,
    }
}

#[test]
fn renders_a_level_with_markers_and_elevation_label() {
    let html = dokime::render(move || view! { <Deck t=t() levels=vec![square(1.5)] /> });
    assert!(html.contains(r#"class="deck""#), "tagged for queries");
    assert!(html.contains("<polygon"), "the footprint is a polygon");
    assert_eq!(
        dokime::count(&html, r#"class="deck-corner""#),
        4,
        "a marker per corner"
    );
    assert!(html.contains("+1.5 ft"), "the elevation label");
}

#[test]
fn renders_one_polygon_per_level() {
    let html =
        dokime::render(move || view! { <Deck t=t() levels=vec![square(0.5), square(2.0)] /> });
    assert_eq!(dokime::count(&html, "<polygon"), 2, "one polygon per level");
    assert!(
        html.contains("+0.5 ft") && html.contains("+2.0 ft"),
        "both labels"
    );
}

#[test]
fn composes_step_runs_with_treads() {
    // A 2 ft drop → 4 steps → 3 interior tread lines.
    let run = StepRun {
        ax: 0.0,
        ay: 0.0,
        bx: 4.0,
        by: 0.0,
        elevation: 2.0,
    };
    let html =
        dokime::render(move || view! { <Deck t=t() levels=vec![square(2.0)] steps=vec![run] /> });
    assert!(html.contains(r#"class="steps""#), "the step run renders");
    assert_eq!(
        dokime::count(&html, r#"class="step-tread""#),
        3,
        "steps-1 interior treads"
    );
}

#[test]
fn no_levels_renders_nothing() {
    let html = dokime::render(move || view! { <Deck t=t() levels=Vec::new() /> });
    assert!(!html.contains(r#"class="deck""#));
}
