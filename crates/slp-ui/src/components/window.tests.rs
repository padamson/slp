//! dokime component test for `Window` (a window opening, in px space).

use leptos::prelude::*;

use super::Window;

#[test]
fn renders_a_window_as_a_framed_glass_slice() {
    let html = dokime::render(|| view! { <Window x1=0.0 y1=0.0 x2=8.0 y2=0.0 /> });
    assert!(html.contains(r#"class="window""#), "tagged as a window");
    assert!(
        html.contains(r#"class="window-glass""#),
        "draws the glass pane"
    );
    // The frame box (two faces + two jambs) is what makes it read as a window.
    assert_eq!(
        dokime::count(&html, r#"class="window-frame""#),
        4,
        "frame box: two wall faces + two jambs"
    );
}

#[test]
fn a_zero_length_span_falls_back_to_a_fixed_normal_not_nan() {
    let html = dokime::render(|| view! { <Window x1=5.0 y1=5.0 x2=5.0 y2=5.0 /> });
    assert!(
        !html.contains("NaN"),
        "a zero-length span must not divide by zero into NaN coordinates"
    );
    assert_eq!(
        dokime::count(&html, r#"class="window-frame""#),
        4,
        "still draws the frame box for a degenerate span"
    );
}
