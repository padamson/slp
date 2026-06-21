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
