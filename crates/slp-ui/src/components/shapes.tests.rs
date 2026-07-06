//! dokime component tests for `Shapes` (drawn paver/mulch/… areas).

use leptos::prelude::*;
use slp_core::{Coord, Shape};

use super::{Shapes, Transform};

fn t() -> Transform {
    Transform {
        px_ft: 10.0,
        pad: 0.0,
        yard_d: 20.0,
    }
}

fn square(elevation: f64) -> Shape {
    Shape {
        corners: vec![
            Coord::new(0.0, 0.0),
            Coord::new(4.0, 0.0),
            Coord::new(4.0, 3.0),
            Coord::new(0.0, 3.0),
        ],
        elevation,
        bulges: Vec::new(),
    }
}

#[test]
fn renders_a_shape_with_markers_and_area_label() {
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![square(0.0)] /> });
    assert!(html.contains(r#"class="shapes""#), "tagged for queries");
    assert!(html.contains("<polygon"), "the area is a polygon");
    assert_eq!(
        dokime::count(&html, r#"class="shape-corner""#),
        4,
        "a marker per corner"
    );
    // 4 ft x 3 ft = 12 ft², elevation 0 -> no elevation suffix.
    assert!(html.contains(">12 ft²<"), "the bare area label, no suffix");
}

#[test]
fn a_nonzero_elevation_appends_to_the_label() {
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![square(1.5)] /> });
    assert!(html.contains("12 ft² · +1.5 ft"), "area + elevation label");
}

#[test]
fn a_negative_elevation_shows_a_single_minus_sign() {
    // A below-grade area (e.g. a sunken paver patio) reads "-0.5 ft", not the
    // doubled-up "+-0.5 ft" a naive `+{elevation}` format would produce.
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![square(-0.5)] /> });
    assert!(html.contains("12 ft² · -0.5 ft"), "single minus sign");
    assert!(!html.contains("+-"), "never a doubled sign");
}

#[test]
fn renders_one_polygon_per_shape() {
    let html =
        dokime::render(move || view! { <Shapes t=t() shapes=vec![square(0.0), square(2.0)] /> });
    assert_eq!(dokime::count(&html, "<polygon"), 2, "one polygon per shape");
}

#[test]
fn no_shapes_renders_nothing() {
    let html = dokime::render(move || view! { <Shapes t=t() shapes=Vec::new() /> });
    assert!(!html.contains(r#"class="shapes""#));
}

#[test]
fn a_bowed_edge_renders_a_path_with_an_arc_command_not_a_polygon() {
    // Bow the first edge (0,0)->(4,0) into an arc — the whole boundary becomes
    // a <path> with an `A` arc command, no <polygon>.
    let mut s = square(0.0);
    s.bulges = vec![0.5, 0.0, 0.0, 0.0];
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![s] /> });
    assert!(!html.contains("<polygon"), "an arced boundary is a path");
    assert!(
        html.contains("<path"),
        "the arced boundary renders as a path"
    );
    // The path has exactly one arc command (the one bowed edge).
    assert_eq!(dokime::count(&html, " A "), 1, "one arc command");
}

#[test]
fn a_bowed_edge_changes_the_reported_area() {
    // Bowing an edge outward (negative bulge = away from a CCW interior) grows
    // the area past the straight 12 ft²; the label reflects it.
    let mut s = square(0.0);
    s.bulges = vec![-1.0, 0.0, 0.0, 0.0]; // bottom edge bows out into a semicircle
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![s] /> });
    assert!(
        !html.contains(">12 ft²<"),
        "the area is no longer the straight 12"
    );
}

#[test]
fn skips_a_degenerate_shape_with_too_few_corners() {
    let degenerate = Shape {
        corners: vec![Coord::new(5.0, 5.0), Coord::new(6.0, 5.0)],
        elevation: 0.0,
        bulges: Vec::new(),
    };
    let html = dokime::render(move || {
        view! { <Shapes t=t() shapes=vec![square(0.0), degenerate] /> }
    });
    assert_eq!(
        dokime::count(&html, "<polygon"),
        1,
        "a shape with under 3 corners has no area to render"
    );
}

#[test]
fn an_unselected_shape_has_plain_corner_markers() {
    let html = dokime::render(move || view! { <Shapes t=t() shapes=vec![square(0.0)] /> });
    assert_eq!(dokime::count(&html, r#"class="shape-corner""#), 4);
    assert_eq!(dokime::count(&html, r#"data-testid="shape-node""#), 0);
    assert!(!html.contains(r#"class="shape shape--selected""#));
}

#[test]
fn a_selected_shape_shows_interactive_node_handles_instead() {
    let html = dokime::render(
        move || view! { <Shapes t=t() shapes=vec![square(0.0)] selected=Some(0) /> },
    );
    assert!(html.contains(r#"class="shape shape--selected""#));
    assert_eq!(dokime::count(&html, r#"class="shape-corner""#), 0);
    assert_eq!(dokime::count(&html, r#"data-testid="shape-node""#), 4);
}

#[test]
fn a_selected_shape_shows_a_bulge_handle_per_edge() {
    // The square has 4 edges, so 4 edge (bulge) handles when selected — none
    // when unselected.
    let selected = dokime::render(
        move || view! { <Shapes t=t() shapes=vec![square(0.0)] selected=Some(0) /> },
    );
    assert_eq!(
        dokime::count(&selected, r#"data-testid="shape-edge-handle""#),
        4
    );
    let plain = dokime::render(move || view! { <Shapes t=t() shapes=vec![square(0.0)] /> });
    assert_eq!(
        dokime::count(&plain, r#"data-testid="shape-edge-handle""#),
        0
    );
}

#[test]
fn only_the_selected_shape_gets_node_handles() {
    let html = dokime::render(move || {
        view! { <Shapes t=t() shapes=vec![square(0.0), square(2.0)] selected=Some(1) /> }
    });
    // One shape (index 0) stays plain; the other (index 1) gets node handles.
    assert_eq!(dokime::count(&html, r#"class="shape-corner""#), 4);
    assert_eq!(dokime::count(&html, r#"data-testid="shape-node""#), 4);
}

#[test]
fn no_popup_with_fewer_than_two_selected_nodes() {
    let html = dokime::render(move || {
        view! { <Shapes t=t() shapes=vec![square(0.0)] selected=Some(0) selected_nodes=vec![1] /> }
    });
    assert!(!html.contains("node-insert-popup"));
}

#[test]
fn a_pair_of_selected_nodes_shows_the_insert_cancel_popup() {
    let html = dokime::render(move || {
        view! {
            <Shapes t=t() shapes=vec![square(0.0)] selected=Some(0) selected_nodes=vec![0, 1] />
        }
    });
    assert!(html.contains("node-insert-popup"));
    assert!(html.contains(r#"data-testid="insert-node""#));
    assert!(html.contains(r#"data-testid="cancel-node-select""#));
}
