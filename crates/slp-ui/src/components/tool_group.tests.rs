//! dokime component test for `ToolGroup`.

use leptos::prelude::*;

use super::{ToolButton, ToolGroup};

#[test]
fn renders_a_label_and_its_children() {
    let html = dokime::render(|| {
        view! {
            <ToolGroup label="House">
                <ToolButton
                    label="Draw house"
                    testid="draw-house"
                    active=Signal::derive(|| false)
                    on_pick=Callback::new(|()| {})
                />
            </ToolGroup>
        }
    });
    assert!(
        html.contains(r#"class="tool-group-label""#),
        "group heading"
    );
    assert!(html.contains("House"), "group label");
    assert!(
        html.contains("Draw house"),
        "children render inside the group"
    );
}
