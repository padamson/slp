//! dokime component tests for `ToolButton`.

use leptos::prelude::*;

use super::ToolButton;

#[test]
fn renders_label_testid_and_active_class() {
    let html = dokime::render(|| {
        view! {
            <ToolButton
                label="Draw house"
                testid="draw-house"
                active=Signal::derive(|| true)
                on_pick=Callback::new(|()| {})
            />
        }
    });
    assert!(html.contains("Draw house"), "label");
    assert!(html.contains(r#"data-testid="draw-house""#), "testid");
    assert!(html.contains("active"), "active class when active");
}

#[test]
fn no_active_class_when_inactive() {
    let html = dokime::render(|| {
        view! {
            <ToolButton
                label="X"
                testid="x"
                active=Signal::derive(|| false)
                on_pick=Callback::new(|()| {})
            />
        }
    });
    assert!(!html.contains("active"), "no active class when inactive");
}

#[test]
fn renders_disabled_with_a_title() {
    let html = dokime::render(|| {
        view! {
            <ToolButton
                label="Save As*"
                testid="save-plan-as"
                active=Signal::derive(|| false)
                on_pick=Callback::new(|()| {})
                disabled=true
                title="only in Chrome"
            />
        }
    });
    assert!(html.contains("disabled"), "the button is disabled");
    assert!(html.contains(r#"title="only in Chrome""#), "the tooltip");
}

#[test]
fn is_not_disabled_by_default() {
    let html = dokime::render(|| {
        view! {
            <ToolButton
                label="X"
                testid="x"
                active=Signal::derive(|| false)
                on_pick=Callback::new(|()| {})
            />
        }
    });
    assert!(!html.contains("disabled"), "not disabled unless asked");
}
