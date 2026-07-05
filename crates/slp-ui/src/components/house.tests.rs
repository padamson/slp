//! dokime component tests for `House` (render the outline + composed openings).

use leptos::prelude::*;
use slp_core::{Coord, Opening, OpeningKind};

use super::{House, Transform};

fn t() -> Transform {
    Transform {
        px_ft: 10.0,
        pad: 0.0,
        yard_d: 10.0,
    }
}

#[test]
fn renders_the_outline_as_a_scaled_polygon() {
    let corners = vec![
        Coord::new(0.0, 0.0),
        Coord::new(4.0, 0.0),
        Coord::new(4.0, 3.0),
        Coord::new(0.0, 3.0),
    ];
    let html = dokime::render(move || view! { <House t=t() corners=corners /> });

    assert!(html.contains("<polygon"), "the outline is an SVG polygon");
    assert!(
        html.contains(r#"class="house""#),
        "tagged for styling/queries"
    );
    // SW corner (0,0) → screen (0, 100); NE corner (4,3) → (40, 70).
    assert!(
        html.contains("0,100"),
        "south-west corner is placed to scale"
    );
    assert!(
        html.contains("40,70"),
        "north-east corner is placed to scale"
    );
}

#[test]
fn no_corners_renders_nothing() {
    let html = dokime::render(move || view! { <House t=t() corners=Vec::new() /> });
    assert!(
        !html.contains("<polygon") && !html.contains(r#"class="house""#),
        "an empty house draws nothing (no stray outline)"
    );
}

#[test]
fn composes_openings_onto_the_walls() {
    // A square house with a door on wall 0 and a window on wall 1.
    let corners = vec![
        Coord::new(0.0, 0.0),
        Coord::new(10.0, 0.0),
        Coord::new(10.0, 10.0),
        Coord::new(0.0, 10.0),
    ];
    let openings = vec![
        Opening::new(OpeningKind::door, 3.0, 0, 3.0),
        Opening::new(OpeningKind::window, 3.0, 1, 3.0),
    ];
    let html = dokime::render(move || view! { <House t=t() corners=corners openings=openings /> });
    assert!(
        html.contains(r#"class="door""#),
        "the door renders on its wall"
    );
    assert!(
        html.contains(r#"class="window""#),
        "the window renders on its wall"
    );
}

#[test]
fn one_corner_shows_a_marker_but_no_outline() {
    let corners = vec![Coord::new(1.0, 1.0)];
    let html = dokime::render(move || view! { <House t=t() corners=corners /> });
    assert!(html.contains("<circle"), "the first corner shows a marker");
    assert!(
        !html.contains("<polygon"),
        "a single point is not yet an outline"
    );
}

#[test]
fn two_corners_draw_one_wall_not_a_closed_ring() {
    // An open chain: the single edge between the two corners, no wrap-back.
    let corners = vec![Coord::new(0.0, 0.0), Coord::new(4.0, 0.0)];
    let html = dokime::render(move || view! { <House t=t() corners=corners /> });
    assert_eq!(
        dokime::count(&html, r#"class="wall""#),
        1,
        "an open 2-corner chain is one wall, not a closed 2-wall ring"
    );
}

fn square() -> Vec<Coord> {
    vec![
        Coord::new(0.0, 0.0),
        Coord::new(4.0, 0.0),
        Coord::new(4.0, 3.0),
        Coord::new(0.0, 3.0),
    ]
}

#[test]
fn an_unselected_house_has_plain_corner_markers() {
    let html = dokime::render(move || view! { <House t=t() corners=square() /> });
    assert_eq!(dokime::count(&html, r#"class="house-corner""#), 4);
    assert_eq!(dokime::count(&html, r#"data-testid="house-node""#), 0);
    assert!(!html.contains("house--selected"));
}

#[test]
fn a_selected_house_shows_interactive_node_handles_instead() {
    let html = dokime::render(move || view! { <House t=t() corners=square() selected=true /> });
    assert!(html.contains("house--selected"));
    assert_eq!(dokime::count(&html, r#"class="house-corner""#), 0);
    assert_eq!(dokime::count(&html, r#"data-testid="house-node""#), 4);
}
