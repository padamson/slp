//! dokime component tests for `Controls` (the args/knobs panel).

use leptos::prelude::*;

use super::Controls;
use crate::ArgControl;

#[test]
fn renders_a_widget_per_arg() {
    let html = dokime::render(|| {
        let args = vec![
            ("active", ArgControl::Bool(RwSignal::new(true))),
            ("width", ArgControl::Num(RwSignal::new(12.0))),
            ("label", ArgControl::Text(RwSignal::new("hi".to_string()))),
        ];
        view! { <Controls args=args /> }
    });
    assert!(html.contains("theoria-controls"), "the panel renders");
    assert!(html.contains("active") && html.contains("width") && html.contains("label"));
    assert_eq!(dokime::count(&html, "<input"), 3, "one widget per arg");
    assert!(html.contains("checkbox"), "bool → checkbox");
    assert!(html.contains("number"), "f64 → number input");
}

#[test]
fn no_args_renders_nothing() {
    let html = dokime::render(|| view! { <Controls args=Vec::new() /> });
    assert!(!html.contains("theoria-controls"));
}
