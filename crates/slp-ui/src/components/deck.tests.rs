//! leptotes component tests for `Deck` (multi-level footprints).

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
    let dom = leptotes::dom(move || view! { <Deck t=t() levels=vec![square(1.5)] /> });

    let level = dom.find("g.deck > g.deck-level");

    assert_eq!(
        (
            level.count("polygon"),
            level.count("circle.deck-corner"),
            level.find("text.deck-label").text()
        ),
        (1, 4, "+1.5 ft".to_string()),
        "{dom}"
    );
}

#[test]
fn a_negative_elevation_shows_a_single_minus_sign() {
    // A below-grade level reads "-0.5 ft", not the doubled-up "+-0.5 ft" a
    // naive `+{elevation}` format would produce.
    let dom = leptotes::dom(move || view! { <Deck t=t() levels=vec![square(-0.5)] /> });

    assert_eq!(dom.find("text.deck-label").text(), "-0.5 ft", "{dom}");
}

#[test]
fn renders_one_polygon_per_level() {
    let dom = leptotes::dom(move || view! { <Deck t=t() levels=vec![square(0.5), square(2.0)] /> });

    assert_eq!(
        (
            dom.count("g.deck-level > polygon"),
            dom.texts("text.deck-label")
        ),
        (2, vec!["+0.5 ft".to_string(), "+2.0 ft".to_string()]),
        "{dom}"
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
    let dom =
        leptotes::dom(move || view! { <Deck t=t() levels=vec![square(2.0)] steps=vec![run] /> });

    assert_eq!(dom.count("g.deck > g.steps line.step-tread"), 3, "{dom}");
}

#[test]
fn no_levels_renders_nothing() {
    let dom = leptotes::dom(move || view! { <Deck t=t() levels=Vec::new() /> });

    assert_eq!(dom.count("*"), 0, "{dom}");
}

#[test]
fn paints_lower_elevations_first_so_higher_layers_on_top() {
    // Passed HIGH-then-LOW: the render still paints LOW first, per the sort
    // in `Deck` ("lowest first so higher platforms paint on top").
    let low = square(0.5);
    let high = DeckLevel {
        corners: vec![
            Coord::new(9.0, 9.0),
            Coord::new(13.0, 9.0),
            Coord::new(13.0, 12.0),
            Coord::new(9.0, 12.0),
        ],
        ..DeckLevel::new(2.0)
    };
    let dom = leptotes::dom(move || view! { <Deck t=t() levels=vec![high, low] /> });

    // Document order is paint order. The low level's SW corner is (0,200);
    // the high level's is (90,110).
    let first_corners: Vec<&str> = dom
        .attrs("g.deck-level > polygon", "points")
        .iter()
        .filter_map(|p| p.split_whitespace().next())
        .collect();

    assert_eq!(first_corners, ["0,200", "90,110"], "{dom}");
}

#[test]
fn skips_a_degenerate_level_with_too_few_corners() {
    let degenerate = DeckLevel {
        corners: vec![Coord::new(5.0, 5.0)],
        ..DeckLevel::new(1.0)
    };
    let dom = leptotes::dom(move || view! { <Deck t=t() levels=vec![square(1.0), degenerate] /> });

    // The degenerate level is skipped, not just malformed.
    assert_eq!(dom.count("g.deck-level"), 1, "{dom}");
}

#[test]
fn an_unselected_level_has_plain_corner_markers() {
    let dom = leptotes::dom(move || view! { <Deck t=t() levels=vec![square(1.0)] /> });

    assert_eq!(
        (
            dom.count("circle.deck-corner"),
            dom.count(r#"[data-testid="deck-node"]"#),
            dom.count(".deck-level--selected"),
        ),
        (4, 0, 0),
        "{dom}"
    );
}

#[test]
fn a_selected_level_shows_interactive_node_handles_instead() {
    let dom =
        leptotes::dom(move || view! { <Deck t=t() levels=vec![square(1.0)] selected=Some(0) /> });

    assert_eq!(
        (
            dom.count("g.deck-level.deck-level--selected"),
            dom.count("circle.deck-corner"),
            dom.count(r#"circle[data-testid="deck-node"]"#),
        ),
        (1, 0, 4),
        "{dom}"
    );
}

#[test]
fn selection_addresses_the_original_index_not_the_paint_order() {
    // Passed HIGH-then-LOW (paints low-first, per the sort): `selected=Some(0)`
    // must select the HIGH level (its original index, index 0 as passed in),
    // not whichever level paints first after the sort (which would be LOW).
    let dom = leptotes::dom(move || {
        view! { <Deck t=t() levels=vec![square(2.0), square(0.5)] selected=Some(0) /> }
    });

    let handles: Vec<(String, usize)> = dom
        .find_all("g.deck-level")
        .iter()
        .map(|level| {
            (
                level.find("text.deck-label").text(),
                level.count(r#"[data-testid="deck-node"]"#),
            )
        })
        .collect();

    // Paint order is low first; the handles belong to the high level.
    assert_eq!(
        handles,
        [("+0.5 ft".to_string(), 0), ("+2.0 ft".to_string(), 4)],
        "{dom}"
    );
}
