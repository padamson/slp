//! leptotes component tests for `Grid`.

use leptos::prelude::*;

use super::{Grid, Transform};

#[test]
fn renders_one_line_per_foot_plus_boundary() {
    let t = Transform {
        px_ft: 12.0,
        pad: 40.0,
        yard_d: 10.0,
    };
    let dom = leptotes::dom(move || view! { <Grid t=t yard_w=4.0 yard_d=10.0 /> });

    // (4+1) verticals + (10+1) horizontals = 16 lines.
    assert_eq!(dom.count("g.grid line"), 16, "{dom}");
}

#[test]
fn draws_every_fifth_line_in_the_darker_major_color() {
    let t = Transform {
        px_ft: 12.0,
        pad: 40.0,
        yard_d: 5.0,
    };
    let dom = leptotes::dom(move || view! { <Grid t=t yard_w=10.0 yard_d=5.0 /> });

    // verticals 0..=10: majors at 0,5,10 (3); horizontals 0..=5: majors at
    // 0,5 (2). The other 8 verticals and 4 horizontals are the faint minors.
    assert_eq!(
        (
            dom.count(r##"line[stroke="#0000001c"]"##),
            dom.count(r##"line[stroke="#0000000d"]"##)
        ),
        (5, 12),
        "{dom}"
    );
}
