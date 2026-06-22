//! dokime component tests for `House` (H1.0 — render the outline to scale).

use leptos::prelude::*;
use slp_core::Coord;

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
fn no_corners_renders_no_polygon() {
    let html = dokime::render(move || view! { <House t=t() corners=Vec::new() /> });
    assert!(
        !html.contains("<polygon"),
        "an empty house draws nothing (no stray outline)"
    );
}
