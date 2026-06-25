//! dokime component tests for the `Placement` overlay.

use leptos::prelude::*;
use slp_core::Coord;

use super::{Placement, Transform};

fn t() -> Transform {
    Transform {
        px_ft: 10.0,
        pad: 0.0,
        yard_d: 10.0,
    }
}

#[test]
fn nothing_placed_and_no_preview_renders_nothing() {
    let html = dokime::render(move || view! { <Placement t=t() placed=Vec::new() /> });
    assert!(!html.contains(r#"class="placement""#));
}

#[test]
fn shows_nodes_a_chain_and_a_rubber_band_to_the_preview() {
    let placed = vec![Coord::new(0.0, 0.0), Coord::new(4.0, 0.0)];
    let html = dokime::render(move || {
        view! { <Placement t=t() placed=placed preview=Some(Coord::new(4.0, 3.0)) /> }
    });
    assert_eq!(
        dokime::count(&html, r#"class="placement-node""#),
        2,
        "a marker per node"
    );
    assert!(html.contains("<polyline"), "the chain through placed nodes");
    assert!(
        html.contains(r#"class="placement-band""#),
        "the rubber-band to the previewed node"
    );
}
