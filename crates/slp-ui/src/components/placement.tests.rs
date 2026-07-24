//! dokime component tests for the `Placement` overlay.

use leptos::prelude::*;
use slp_core::Coord;

use super::{Footprint, Placement, Transform};

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

#[test]
fn one_placed_node_shows_the_marker_and_band_but_no_chain() {
    let placed = vec![Coord::new(0.0, 0.0)];
    let html = dokime::render(move || {
        view! { <Placement t=t() placed=placed preview=Some(Coord::new(4.0, 3.0)) /> }
    });
    assert_eq!(
        dokime::count(&html, r#"class="placement-node""#),
        1,
        "a marker for the one placed node"
    );
    assert!(
        html.contains(r#"class="placement-band""#),
        "the rubber-band to the previewed node"
    );
    assert!(
        !html.contains("<polyline"),
        "no chain with fewer than 2 placed nodes"
    );
}

#[test]
fn no_object_footprint_shows_the_plain_hollow_node_marker() {
    let html = dokime::render(move || {
        view! { <Placement t=t() placed=Vec::new() preview=Some(Coord::new(4.0, 3.0)) /> }
    });
    assert!(
        !html.contains(r#"class="placement-object-preview""#),
        "no shape preview without an armed item"
    );
    // The plain marker is a hollow (fill="none") circle.
    assert!(html.contains(r#"fill="none""#), "the plain hollow marker");
}

#[test]
fn an_armed_round_item_previews_a_translucent_circle() {
    // A ⌀4 ft fire pit: 10 px/ft → radius 20 px.
    let fp = Footprint {
        w_ft: 4.0,
        d_ft: 4.0,
        circle: true,
        clearance_ft: None,
        category: None,
        trunk_ft: None,
        slab_overhang_ft: None,
    };
    let html = dokime::render(move || {
        view! {
            <Placement
                t=t()
                placed=Vec::new()
                preview=Some(Coord::new(4.0, 3.0))
                object_footprint=Some(fp)
            />
        }
    });
    assert!(
        html.contains(r#"class="placement-object-preview""#),
        "the shape-aware preview wrapper"
    );
    assert!(html.contains("<circle"), "a round item previews a circle");
    assert!(html.contains(r#"r="20""#), "4 ft diameter → 20 px radius");
    assert!(html.contains(r#"opacity="0.5""#), "faint (~50%) preview");
}

#[test]
fn an_armed_rect_item_previews_a_translucent_rect() {
    // A 3×1.5 ft chair: 10 px/ft → 30 × 15 px.
    let fp = Footprint {
        w_ft: 3.0,
        d_ft: 1.5,
        circle: false,
        clearance_ft: None,
        category: None,
        trunk_ft: None,
        slab_overhang_ft: None,
    };
    let html = dokime::render(move || {
        view! {
            <Placement
                t=t()
                placed=Vec::new()
                preview=Some(Coord::new(4.0, 3.0))
                object_footprint=Some(fp)
            />
        }
    });
    assert!(html.contains("<rect"), "a rectangular item previews a rect");
    assert!(html.contains(r#"width="30""#), "3 ft → 30 px wide");
    assert!(html.contains(r#"height="15""#), "1.5 ft → 15 px deep");
    assert!(html.contains(r#"opacity="0.5""#), "faint (~50%) preview");
}
