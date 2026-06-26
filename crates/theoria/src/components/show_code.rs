//! "Show code": a collapsible block of a story's captured source (from the
//! `#[story]` macro).

use leptos::prelude::*;

#[component]
pub fn ShowCode(source: &'static str) -> impl IntoView {
    view! {
        <details class="theoria-code">
            <summary>"Show code"</summary>
            <pre>
                <code>{source}</code>
            </pre>
        </details>
    }
}
