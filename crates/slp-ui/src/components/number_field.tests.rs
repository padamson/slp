//! dokime component test for `NumberField`.

use leptos::prelude::*;

use super::NumberField;

#[test]
fn renders_a_labeled_number_input() {
    let html = dokime::render(|| {
        view! {
            <NumberField
                label="Width (ft)"
                testid="yard-width"
                value=Signal::derive(|| 12.0)
                on_input=Callback::new(|_v| {})
                step=0.5
            />
        }
    });
    assert!(html.contains("Width (ft)"), "label");
    assert!(html.contains(r#"data-testid="yard-width""#), "testid");
    assert!(html.contains("number"), "is a number input");
}
