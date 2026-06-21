//! dokime component tests for `Yard`.

use leptos::prelude::*;

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
