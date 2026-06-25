//! A labeled cluster of toolbar controls — the layout primitive that groups
//! related buttons/fields/toggles (e.g. "House", "Deck", "Snap") under a heading.

use leptos::prelude::*;

#[component]
pub fn ToolGroup(label: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="tool-group">
            <span class="tool-group-label">{label}</span>
            <div class="tool-group-items">{children()}</div>
        </div>
    }
}
