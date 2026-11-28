//! leptotes component test for `Window` (a window opening, in px space).

use leptos::prelude::*;

use super::Window;

#[test]
fn renders_a_window_as_a_framed_glass_slice() {
    let dom = leptotes::dom(|| view! { <Window x1=0.0 y1=0.0 x2=8.0 y2=0.0 /> });

    let window = dom.find("g.window");

    // The frame box (two faces + two jambs) is what makes it read as a window.
    assert_eq!(
        (
            window.count("line.window-glass"),
            window.count("line.window-frame")
        ),
        (1, 4),
        "{dom}"
    );
}

#[test]
fn a_zero_length_span_falls_back_to_a_fixed_normal_not_nan() {
    let dom = leptotes::dom(|| view! { <Window x1=5.0 y1=5.0 x2=5.0 y2=5.0 /> });

    let coords: Vec<&str> = ["x1", "y1", "x2", "y2"]
        .iter()
        .flat_map(|name| dom.attrs("line", name))
        .collect();

    assert!(
        coords.len() == 24
            && coords
                .iter()
                .all(|v| v.parse::<f64>().is_ok_and(f64::is_finite)),
        "six finite lines, no division by zero into NaN; got {coords:?} in\n{dom}"
    );
}

#[test]
fn a_zero_length_span_still_draws_the_frame_box() {
    let dom = leptotes::dom(|| view! { <Window x1=5.0 y1=5.0 x2=5.0 y2=5.0 /> });

    assert_eq!(dom.count("g.window line.window-frame"), 4, "{dom}");
}
