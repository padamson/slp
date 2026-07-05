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

#[test]
fn renders_the_step_attribute() {
    let html = dokime::render(|| {
        view! {
            <NumberField
                label="Width (ft)"
                testid="yard-width"
                value=Signal::derive(|| 12.0)
                on_input=Callback::new(|_v| {})
                step=0.1
            />
        }
    });
    assert!(html.contains(r#"step="0.1""#), "step");
}

#[test]
fn renders_the_min_attribute_when_set() {
    let html = dokime::render(|| {
        view! {
            <NumberField
                label="Width (ft)"
                testid="yard-width"
                value=Signal::derive(|| 12.0)
                on_input=Callback::new(|_v| {})
                step=0.5
                min=1.0
            />
        }
    });
    assert!(html.contains(r#"min="1""#), "min");
}

#[test]
fn omits_the_min_attribute_when_unset() {
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
    assert!(!html.contains("min="), "no min attribute when unset");
}
