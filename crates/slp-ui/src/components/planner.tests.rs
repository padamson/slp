//! dokime component test for `Planner` (the composed root UI), plus unit tests
//! for the pure click-classification used while drawing the house outline.

use leptos::prelude::*;
use slp_core::Coord;

use super::Planner;
use super::planner::{Pick, classify_pick};

#[test]
fn clicks_add_corners_until_snap_closes_the_ring() {
    let snap = 2.0;
    // First clicks always add (an empty/partial ring can't be closed).
    assert_eq!(
        classify_pick(&[], Coord::new(0.0, 0.0), snap),
        Pick::Add(Coord::new(0.0, 0.0))
    );
    let two = [Coord::new(0.0, 0.0), Coord::new(10.0, 0.0)];
    // Near the start but only two corners so far → still an add (need ≥3).
    assert_eq!(
        classify_pick(&two, Coord::new(0.5, 0.5), snap),
        Pick::Add(Coord::new(0.5, 0.5))
    );
    let three = [
        Coord::new(0.0, 0.0),
        Coord::new(10.0, 0.0),
        Coord::new(10.0, 8.0),
    ];
    // ≥3 corners and a click within snap of the first → close.
    assert_eq!(
        classify_pick(&three, Coord::new(0.5, 0.5), snap),
        Pick::Close
    );
    // ≥3 corners but the click is far from the start → keep adding.
    assert_eq!(
        classify_pick(&three, Coord::new(5.0, 5.0), snap),
        Pick::Add(Coord::new(5.0, 5.0))
    );
}

#[test]
fn renders_header_controls_and_yard() {
    let html = dokime::render(|| view! { <Planner /> });
    assert!(
        html.contains("Simple Landscape Planner"),
        "the header renders"
    );
    assert_eq!(
        dokime::count(&html, "<input"),
        2,
        "the two yard-size inputs"
    );
    assert!(
        html.contains(r#"data-testid="yard""#),
        "the yard canvas renders"
    );
}
