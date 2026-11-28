//! leptotes component tests for `Wall` (composes a wall's doors/windows).

use leptos::prelude::*;
use slp_core::{Coord, Opening, OpeningKind};

use super::{Transform, Wall};

fn t() -> Transform {
    Transform {
        px_ft: 10.0,
        pad: 0.0,
        yard_d: 10.0,
    }
}

#[test]
fn composes_a_door_and_a_window() {
    // Opening::new(kind, offset, wall, width)
    let openings = vec![
        Opening::new(OpeningKind::door, 1.0, 0, 3.0),
        Opening::new(OpeningKind::window, 6.0, 0, 2.0),
    ];
    let dom = leptotes::dom(move || {
        view! { <Wall t=t() start=Coord::new(0.0, 0.0) end=Coord::new(10.0, 0.0) openings=openings /> }
    });

    let wall = dom.find("g.wall");

    assert_eq!(
        (wall.count("g.door"), wall.count("g.window")),
        (1, 1),
        "{dom}"
    );
}

#[test]
fn draws_its_edge_even_without_openings() {
    let dom = leptotes::dom(move || {
        view! { <Wall t=t() start=Coord::new(0.0, 0.0) end=Coord::new(5.0, 0.0) openings=Vec::new() /> }
    });

    let wall = dom.find("g.wall");

    assert_eq!(
        (wall.count("line.wall-edge"), wall.count("g.door, g.window")),
        (1, 0),
        "{dom}"
    );
}
