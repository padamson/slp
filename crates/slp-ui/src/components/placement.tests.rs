//! leptotes component tests for the `Placement` overlay.

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

fn footprint(w_ft: f64, d_ft: f64, circle: bool) -> Footprint {
    Footprint {
        w_ft,
        d_ft,
        circle,
        clearance_ft: None,
        category: None,
        trunk_ft: None,
        slab_overhang_ft: None,
    }
}

fn armed(fp: Footprint) -> leptotes::Dom {
    leptotes::dom(move || {
        view! {
            <Placement
                t=t()
                placed=Vec::new()
                preview=Some(Coord::new(4.0, 3.0))
                object_footprint=Some(fp)
            />
        }
    })
}

#[test]
fn nothing_placed_and_no_preview_renders_nothing() {
    let dom = leptotes::dom(move || view! { <Placement t=t() placed=Vec::new() /> });

    assert_eq!(dom.count("*"), 0, "{dom}");
}

#[test]
fn shows_nodes_a_chain_and_a_rubber_band_to_the_preview() {
    let placed = vec![Coord::new(0.0, 0.0), Coord::new(4.0, 0.0)];
    let dom = leptotes::dom(move || {
        view! { <Placement t=t() placed=placed preview=Some(Coord::new(4.0, 3.0)) /> }
    });

    let overlay = dom.find("g.placement");

    assert_eq!(
        (
            overlay.count("circle.placement-node"),
            overlay.count("polyline"),
            overlay.count("line.placement-band"),
        ),
        (2, 1, 1),
        "{dom}"
    );
}

#[test]
fn one_placed_node_shows_the_marker_and_band_but_no_chain() {
    let placed = vec![Coord::new(0.0, 0.0)];
    let dom = leptotes::dom(move || {
        view! { <Placement t=t() placed=placed preview=Some(Coord::new(4.0, 3.0)) /> }
    });

    assert_eq!(
        (
            dom.count("circle.placement-node"),
            dom.count("line.placement-band"),
            dom.count("polyline"),
        ),
        (1, 1, 0),
        "{dom}"
    );
}

#[test]
fn no_object_footprint_shows_the_plain_hollow_node_marker() {
    let dom = leptotes::dom(move || {
        view! { <Placement t=t() placed=Vec::new() preview=Some(Coord::new(4.0, 3.0)) /> }
    });

    let preview = dom.find("g.placement-preview");

    assert_eq!(
        (
            preview.count(".placement-object-preview"),
            preview.find("circle").attr("fill")
        ),
        (0, Some("none")),
        "{dom}"
    );
}

#[test]
fn an_armed_round_item_previews_a_translucent_circle() {
    // A ⌀4 ft fire pit: 10 px/ft → radius 20 px, at a faint (~50%) opacity.
    let dom = armed(footprint(4.0, 4.0, true));

    let preview = dom.find("g.placement-object-preview");

    assert_eq!(
        (preview.attr("opacity"), preview.find("circle").attr("r")),
        (Some("0.5"), Some("20")),
        "{dom}"
    );
}

#[test]
fn an_armed_rect_item_previews_a_translucent_rect() {
    // A 3×1.5 ft chair: 10 px/ft → 30 × 15 px, at a faint (~50%) opacity.
    let dom = armed(footprint(3.0, 1.5, false));

    let preview = dom.find("g.placement-object-preview");
    let rect = preview.find("rect");

    assert_eq!(
        (
            preview.attr("opacity"),
            rect.attr("width"),
            rect.attr("height")
        ),
        (Some("0.5"), Some("30"), Some("15")),
        "{dom}"
    );
}
