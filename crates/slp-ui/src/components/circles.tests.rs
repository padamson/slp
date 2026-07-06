//! dokime component tests for `Circles` (standalone round drawn areas).

use leptos::prelude::*;
use slp_core::{Circle, Coord};

use super::{Circles, Transform};

fn t() -> Transform {
    Transform {
        px_ft: 10.0,
        pad: 0.0,
        yard_d: 20.0,
    }
}

fn circle(elevation: f64) -> Circle {
    Circle {
        center: Box::new(Coord::new(5.0, 5.0)),
        elevation,
        radius_ft: 2.0,
    }
}

#[test]
fn renders_a_circle_with_area_diameter_and_no_elevation_suffix() {
    let html = dokime::render(move || view! { <Circles t=t() circles=vec![circle(0.0)] /> });
    assert!(html.contains(r#"class="circles""#), "tagged for queries");
    assert!(html.contains("<circle"), "the area is a circle");
    // radius 2 ft -> area = π·4 ≈ 13 ft², diameter 4 ft.
    assert!(
        html.contains("13 ft² · ⌀4 ft"),
        "area + diameter, no suffix"
    );
    assert!(!html.contains("ft · +"), "no elevation suffix at grade");
}

#[test]
fn a_nonzero_elevation_appends_to_the_label() {
    let html = dokime::render(move || view! { <Circles t=t() circles=vec![circle(1.5)] /> });
    assert!(html.contains("13 ft² · ⌀4 ft · +1.5 ft"));
}

#[test]
fn a_negative_elevation_shows_a_single_minus_sign() {
    let html = dokime::render(move || view! { <Circles t=t() circles=vec![circle(-0.5)] /> });
    assert!(
        html.contains("13 ft² · ⌀4 ft · -0.5 ft"),
        "single minus sign"
    );
    assert!(!html.contains("+-"), "never a doubled sign");
}

#[test]
fn renders_one_circle_per_entry() {
    let html = dokime::render(move || {
        view! { <Circles t=t() circles=vec![circle(0.0), circle(2.0)] /> }
    });
    // 2 filled circles + no handles (unselected) = 2 <circle> elements.
    assert_eq!(dokime::count(&html, "<circle"), 2);
}

#[test]
fn no_circles_renders_nothing() {
    let html = dokime::render(move || view! { <Circles t=t() circles=Vec::new() /> });
    assert!(!html.contains(r#"class="circles""#));
}

#[test]
fn an_unselected_circle_has_no_resize_handle() {
    let html = dokime::render(move || view! { <Circles t=t() circles=vec![circle(0.0)] /> });
    assert_eq!(
        dokime::count(&html, r#"data-testid="circle-resize-handle""#),
        0
    );
    assert!(!html.contains("circle-area--selected"));
}

#[test]
fn a_selected_circle_shows_a_resize_handle() {
    let html = dokime::render(move || {
        view! { <Circles t=t() circles=vec![circle(0.0)] selected=Some(0) /> }
    });
    assert!(html.contains("circle-area--selected"));
    assert_eq!(
        dokime::count(&html, r#"data-testid="circle-resize-handle""#),
        1
    );
}

#[test]
fn only_the_selected_circle_gets_a_handle() {
    let html = dokime::render(move || {
        view! { <Circles t=t() circles=vec![circle(0.0), circle(0.0)] selected=Some(1) /> }
    });
    assert_eq!(
        dokime::count(&html, r#"data-testid="circle-resize-handle""#),
        1
    );
}
