//! dokime component tests for `Deck`.

use leptos::prelude::*;
use slp_core::Coord;

use super::{Deck, Transform};

fn t() -> Transform {
    Transform {
        px_ft: 10.0,
        pad: 0.0,
        yard_d: 10.0,
    }
}

#[test]
fn renders_the_footprint_polygon() {
    let corners = vec![
        Coord::new(0.0, 0.0),
        Coord::new(4.0, 0.0),
        Coord::new(4.0, 3.0),
        Coord::new(0.0, 3.0),
    ];
    let html = dokime::render(move || view! { <Deck t=t() corners=corners /> });
    assert!(html.contains(r#"class="deck""#), "tagged for queries");
    assert!(html.contains("<polygon"), "the footprint is a polygon");
    assert_eq!(
        dokime::count(&html, r#"class="deck-corner""#),
        4,
        "a marker per corner"
    );
}

#[test]
fn no_corners_renders_nothing() {
    let html = dokime::render(move || view! { <Deck t=t() corners=Vec::new() /> });
    assert!(!html.contains(r#"class="deck""#));
}
