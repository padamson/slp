//! dokime component test for `TextField`.

use leptos::prelude::*;

use super::TextField;

#[test]
fn renders_a_labeled_text_input_with_its_value() {
    let html = dokime::render(|| {
        view! {
            <TextField
                label="Name"
                testid="catalog-name"
                value=Signal::derive(|| "Lounge chair".to_string())
                on_input=Callback::new(|_v| {})
            />
        }
    });
    assert!(html.contains("Name"), "label");
    assert!(html.contains(r#"data-testid="catalog-name""#), "testid");
    assert!(html.contains(r#"type="text""#), "is a text input");
    assert!(html.contains("Lounge chair"), "shows the current value");
}

#[test]
fn renders_a_placeholder_when_given() {
    let html = dokime::render(|| {
        view! {
            <TextField
                label="Category"
                testid="catalog-category"
                value=Signal::derive(String::new)
                on_input=Callback::new(|_v| {})
                placeholder="e.g. furniture"
            />
        }
    });
    assert!(
        html.contains(r#"placeholder="e.g. furniture""#),
        "placeholder"
    );
}
