//! dokime component tests for `YardControls`.

use leptos::prelude::*;

use super::YardControls;

#[test]
fn renders_width_and_depth_number_inputs() {
    let html = dokime::render(|| {
        let (width, set_width) = signal(70.0_f64);
        let (depth, set_depth) = signal(30.0_f64);
        view! { <YardControls width=width set_width=set_width depth=depth set_depth=set_depth /> }
    });
    assert_eq!(
        dokime::count(&html, "<input"),
        2,
        "a width and a depth input"
    );
    assert!(html.contains(r#"data-testid="yard-width""#));
    assert!(html.contains(r#"data-testid="yard-depth""#));
    assert!(html.contains(r#"type="number""#));
}
