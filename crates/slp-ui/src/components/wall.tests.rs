//! dokime component tests for `Wall` (composes a wall's doors/windows).

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
    let html = dokime::render(move || {
        view! { <Wall t=t() start=Coord::new(0.0, 0.0) end=Coord::new(10.0, 0.0) openings=openings /> }
    });
    assert!(html.contains(r#"class="door""#), "the door is composed in");
    assert!(
        html.contains(r#"class="window""#),
        "the window is composed in"
    );
}

#[test]
fn draws_its_edge_even_without_openings() {
    let html = dokime::render(move || {
        view! { <Wall t=t() start=Coord::new(0.0, 0.0) end=Coord::new(5.0, 0.0) openings=Vec::new() /> }
    });
    assert!(
        html.contains(r#"class="wall-edge""#),
        "a wall draws its own edge line (self-contained)"
    );
    assert!(
        !html.contains(r#"class="door""#) && !html.contains(r#"class="window""#),
        "but no openings are drawn"
    );
}
