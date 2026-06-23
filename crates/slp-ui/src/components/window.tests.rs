//! dokime component test for `Window` (a window opening, in px space).

use leptos::prelude::*;

use super::Window;

#[test]
fn renders_a_window_with_glass() {
    let html = dokime::render(|| view! { <Window x1=0.0 y1=0.0 x2=8.0 y2=0.0 /> });
    assert!(html.contains(r#"class="window""#), "tagged as a window");
    assert!(
        html.contains(r#"class="window-glass""#),
        "draws the glass line"
    );
}
