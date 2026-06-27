//! Render a Markdown string (a story's description, captured from the
//! component's doc-comment) to HTML for autodocs. Headings, lists, emphasis,
//! code, and links are supported; the MDX *authoring format* is not.
//!
//! The input is trusted — it comes from doc-comments in the source, not user
//! input — so the rendered HTML is set directly as `inner_html`.

use leptos::prelude::*;
use pulldown_cmark::{Parser, html};

/// Markdown source → HTML string.
pub fn to_html(md: &str) -> String {
    let mut out = String::new();
    html::push_html(&mut out, Parser::new(md));
    out
}

// `#[prop(into)]` needs an owned `String` (so callers can pass `&str`/`String`);
// we only borrow it to render, hence the allow.
#[allow(clippy::needless_pass_by_value)]
#[component]
pub fn Markdown(#[prop(into)] text: String) -> impl IntoView {
    view! { <div class="theoria-md" inner_html=to_html(&text)></div> }
}
