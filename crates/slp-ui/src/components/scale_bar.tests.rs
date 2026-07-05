//! dokime component tests for `ScaleBar`.

use leptos::prelude::*;

use super::{ScaleBar, Transform};

#[test]
fn renders_label_and_one_line() {
    let t = Transform {
        px_ft: 12.0,
        pad: 40.0,
        yard_d: 30.0,
    };
    let html = dokime::render(move || view! { <ScaleBar t=t baseline_y=400.0 /> });
    assert!(html.contains("10 ft"), "scale bar shows its length label");
    assert_eq!(dokime::count(&html, "<line"), 1, "exactly one bar line");
}

#[test]
fn scales_label_and_line_to_a_non_default_length() {
    let t = Transform {
        px_ft: 12.0,
        pad: 40.0,
        yard_d: 30.0,
    };
    let html = dokime::render(move || view! { <ScaleBar t=t baseline_y=400.0 length_ft=20.0 /> });
    assert!(html.contains("20 ft"), "label reflects the explicit length");
    // x0 = sx(0) = pad = 40; x2 = x0 + 20 ft * 12 px/ft = 280.
    assert!(
        html.contains(r#"x2="280""#),
        "line end scales with length_ft"
    );
}
