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
        ..DeckLevel::new(elevation)
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
fn a_negative_elevation_shows_a_single_minus_sign() {
    // A below-grade level reads "-0.5 ft", not the doubled-up "+-0.5 ft" a
    // naive `+{elevation}` format would produce.
    let html = dokime::render(move || view! { <Deck t=t() levels=vec![square(-0.5)] /> });
    assert!(html.contains("-0.5 ft"), "single minus sign");
    assert!(!html.contains("+-"), "never a doubled sign");
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

#[test]
fn paints_lower_elevations_first_so_higher_layers_on_top() {
    // Passed HIGH-then-LOW: the render still paints LOW first, per the sort
    // in `Deck` ("lowest first so higher platforms paint on top").
    let low = DeckLevel {
        corners: vec![
            Coord::new(0.0, 0.0),
            Coord::new(4.0, 0.0),
            Coord::new(4.0, 3.0),
            Coord::new(0.0, 3.0),
        ],
        ..DeckLevel::new(0.5)
    };
    let high = DeckLevel {
        corners: vec![
            Coord::new(9.0, 9.0),
            Coord::new(13.0, 9.0),
            Coord::new(13.0, 12.0),
            Coord::new(9.0, 12.0),
        ],
        ..DeckLevel::new(2.0)
    };
    let html = dokime::render(move || view! { <Deck t=t() levels=vec![high, low] /> });
    let low_idx = html.find("0,200").expect("low polygon's SW corner renders");
    let high_idx = html
        .find("90,110")
        .expect("high polygon's SW corner renders");
    assert!(low_idx < high_idx, "low paints before high (underneath it)");
}

#[test]
fn skips_a_degenerate_level_with_too_few_corners() {
    let degenerate = DeckLevel {
        corners: vec![Coord::new(5.0, 5.0)],
        ..DeckLevel::new(1.0)
    };
    let html =
        dokime::render(move || view! { <Deck t=t() levels=vec![square(1.0), degenerate] /> });
    assert_eq!(
        dokime::count(&html, "<polygon"),
        1,
        "the degenerate level is skipped, not just malformed"
    );
}
