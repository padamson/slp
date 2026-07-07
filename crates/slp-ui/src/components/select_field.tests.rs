//! dokime component tests for `SelectField`.

use leptos::prelude::*;

use super::SelectField;

fn opts() -> Vec<(String, String)> {
    vec![
        ("per_item".to_string(), "Per item".to_string()),
        ("per_square_foot".to_string(), "Per ft²".to_string()),
        ("per_cubic_yard".to_string(), "Per yd³".to_string()),
    ]
}

#[test]
fn renders_a_labeled_dropdown_with_every_option() {
    let html = dokime::render(|| {
        view! {
            <SelectField
                label="Price unit"
                testid="catalog-price-unit"
                value=Signal::derive(|| "per_square_foot".to_string())
                options=opts()
                on_change=Callback::new(|_| {})
            />
        }
    });
    assert!(html.contains("Price unit"), "label");
    assert!(
        html.contains(r#"data-testid="catalog-price-unit""#),
        "testid"
    );
    assert!(html.contains("Per item"), "an option label");
    assert!(html.contains("Per yd³"), "another option label");
    assert!(
        html.contains(r#"value="per_square_foot""#),
        "an option value"
    );
}

#[test]
fn marks_the_current_value_selected() {
    let html = dokime::render(|| {
        view! {
            <SelectField
                label="Price unit"
                testid="unit"
                value=Signal::derive(|| "per_cubic_yard".to_string())
                options=opts()
                on_change=Callback::new(|_| {})
            />
        }
    });
    // Exactly one option is marked selected — the current value.
    assert_eq!(dokime::count(&html, "selected"), 1, "one selected option");
    assert!(
        html.contains(r#"value="per_cubic_yard" selected"#),
        "the current value is the selected option"
    );
}
