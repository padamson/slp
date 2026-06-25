//! dokime component test for `Toggle`.

use leptos::prelude::*;

use super::Toggle;

#[test]
fn renders_a_labeled_checkbox() {
    let html = dokime::render(|| {
        view! {
            <Toggle
                label="Snap to grid"
                testid="snap-grid"
                checked=Signal::derive(|| true)
                on_toggle=Callback::new(|_b| {})
            />
        }
    });
    assert!(html.contains("Snap to grid"), "label");
    assert!(html.contains(r#"data-testid="snap-grid""#), "testid");
    assert!(html.contains("checkbox"), "is a checkbox");
}
