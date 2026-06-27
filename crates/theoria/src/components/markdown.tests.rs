//! dokime component test for `Markdown`.

use leptos::prelude::*;

use super::Markdown;

#[test]
fn renders_markdown_as_html() {
    let html = dokime::render(|| {
        view! {
            <Markdown text="# Title\n\nSome **bold** text and a list:\n\n- one\n- two" />
        }
    });
    assert!(html.contains("theoria-md"), "tagged for layout/queries");
    assert!(html.contains("<h1"), "# heading → <h1>");
    assert!(
        html.contains("<strong>bold</strong>"),
        "**bold** → <strong>"
    );
    assert!(html.contains("<li>one</li>"), "- item → <li>");
}
