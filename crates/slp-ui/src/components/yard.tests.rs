//! dokime component tests for `Yard`.

use leptos::prelude::*;
use slp_core::Coord;

use super::Yard;

#[test]
fn renders_the_yard_svg_with_scale_bar() {
    let html = dokime::render(|| view! { <Yard yard_w=10.0 yard_d=10.0 px_ft=12.0 pad=40.0 /> });
    assert!(
        html.contains(r#"data-testid="yard""#),
        "the yard canvas is present"
    );
    assert!(
        html.contains("10 ft"),
        "the scale bar renders inside the yard"
    );
}

#[test]
fn renders_the_house_outline_when_given_corners() {
    let house = vec![
        Coord::new(2.0, 2.0),
        Coord::new(6.0, 2.0),
        Coord::new(6.0, 5.0),
        Coord::new(2.0, 5.0),
    ];
    let html = dokime::render(
        move || view! { <Yard yard_w=10.0 yard_d=10.0 px_ft=12.0 pad=40.0 house=house /> },
    );
    assert!(
        html.contains(r#"class="house""#),
        "the house outline draws inside the yard stage"
    );
}

#[test]
fn renders_no_house_outline_by_default() {
    let html = dokime::render(|| view! { <Yard yard_w=10.0 yard_d=10.0 px_ft=12.0 pad=40.0 /> });
    assert!(
        !html.contains(r#"class="house""#),
        "a yard with no house draws no outline"
    );
}
