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

#[test]
fn draws_every_fifth_line_in_the_darker_major_color() {
    let t = Transform {
        px_ft: 12.0,
        pad: 40.0,
        yard_d: 5.0,
    };
    let html = dokime::render(move || view! { <Grid t=t yard_w=10.0 yard_d=5.0 /> });
    // verticals 0..=10: majors at 0,5,10 (3); horizontals 0..=5: majors at 0,5 (2).
    assert_eq!(
        dokime::count(&html, r##"stroke="#0000001c""##),
        5,
        "3 major verticals + 2 major horizontals"
    );
    // the remaining 8 verticals + 4 horizontals are the faint minor lines.
    assert_eq!(
        dokime::count(&html, r##"stroke="#0000000d""##),
        12,
        "8 minor verticals + 4 minor horizontals"
    );
}
