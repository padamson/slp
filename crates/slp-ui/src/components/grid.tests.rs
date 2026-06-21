//! dokime component tests for `Grid`.

use leptos::prelude::*;

use super::{Grid, Transform};

#[test]
fn renders_one_line_per_foot_plus_boundary() {
    let t = Transform {
        px_ft: 12.0,
        pad: 40.0,
        yard_d: 10.0,
    };
    let html = dokime::render(move || view! { <Grid t=t yard_w=4.0 yard_d=10.0 /> });
    // (4+1) verticals + (10+1) horizontals = 16 lines.
    assert_eq!(dokime::count(&html, "<line"), 16);
}
