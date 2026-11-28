//! leptotes component tests for `House` (render the outline + composed openings).

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

fn square() -> Vec<Coord> {
    vec![
        Coord::new(0.0, 0.0),
        Coord::new(4.0, 0.0),
        Coord::new(4.0, 3.0),
        Coord::new(0.0, 3.0),
    ]
}

#[test]
fn renders_the_outline_as_a_scaled_polygon() {
    let dom = leptotes::dom(move || view! { <House t=t() corners=square() /> });

    let points = dom.find("g.house > polygon").attr("points");

    // SW corner (0,0) → screen (0, 100); NE corner (4,3) → (40, 70).
    assert_eq!(points, Some("0,100 40,100 40,70 0,70"), "{dom}");
}

#[test]
fn no_corners_renders_nothing() {
    let dom = leptotes::dom(move || view! { <House t=t() corners=Vec::new() /> });

    assert_eq!(
        dom.count("*"),
        0,
        "an empty house draws nothing; got\n{dom}"
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
    let dom = leptotes::dom(move || view! { <House t=t() corners=corners openings=openings /> });

    let walls = dom.find_all("g.house g.wall");

    assert_eq!(
        (walls[0].count("g.door"), walls[1].count("g.window")),
        (1, 1),
        "{dom}"
    );
}

#[test]
fn one_corner_shows_a_marker_but_no_outline() {
    let corners = vec![Coord::new(1.0, 1.0)];
    let dom = leptotes::dom(move || view! { <House t=t() corners=corners /> });

    assert_eq!(
        (dom.count("circle.house-corner"), dom.count("polygon")),
        (1, 0),
        "{dom}"
    );
}

#[test]
fn two_corners_draw_one_wall_not_a_closed_ring() {
    // An open chain: the single edge between the two corners, no wrap-back.
    let corners = vec![Coord::new(0.0, 0.0), Coord::new(4.0, 0.0)];
    let dom = leptotes::dom(move || view! { <House t=t() corners=corners /> });

    assert_eq!(dom.count("g.wall"), 1, "{dom}");
}

#[test]
fn an_unselected_house_has_plain_corner_markers() {
    let dom = leptotes::dom(move || view! { <House t=t() corners=square() /> });

    assert_eq!(
        (
            dom.count("circle.house-corner"),
            dom.count(r#"[data-testid="house-node"]"#),
            dom.count(".house--selected"),
        ),
        (4, 0, 0),
        "{dom}"
    );
}

#[test]
fn a_selected_house_shows_interactive_node_handles_instead() {
    let dom = leptotes::dom(move || view! { <House t=t() corners=square() selected=true /> });

    assert_eq!(
        (
            dom.count("g.house.house--selected"),
            dom.count("circle.house-corner"),
            dom.count(r#"circle[data-testid="house-node"]"#),
        ),
        (1, 0, 4),
        "{dom}"
    );
}
