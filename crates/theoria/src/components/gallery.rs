//! The gallery: a [`StoryNav`](super::StoryNav) sidebar plus the selected
//! story's view on a stage. The selection is persisted to `localStorage` (by
//! name) so it survives a full page reload — e.g. Trunk's hot reload during
//! development keeps you on the story you were looking at.

use leptos::prelude::*;

use super::{Controls, Markdown, ShowCode, StoryNav};
use crate::Story;

/// `localStorage` key for the selected story name (only used in the browser build).
#[cfg(feature = "csr")]
const STORAGE_KEY: &str = "theoria:selected-story";

#[component]
pub fn Gallery(stories: Vec<Story>) -> impl IntoView {
    let names: Vec<&'static str> = stories.iter().map(|s| s.name).collect();

    // Restore the previously-selected story (by name) if it still exists.
    let initial = load_selected()
        .and_then(|n| names.iter().position(|&name| name == n.as_str()))
        .unwrap_or(0);
    let (selected, set_selected) = signal(initial);

    // Persist the selection whenever it changes (no-op under ssr / in tests).
    let names_for_save = names.clone();
    Effect::new(move |_| {
        if let Some(name) = names_for_save.get(selected.get()) {
            save_selected(name);
        }
    });

    // Keep `stories` reachable from several reactive closures (stage, controls,
    // docs, show-code) — `StoredValue` is `Copy` and read-shared.
    let stories = StoredValue::new(stories);

    let stage = move || stories.with_value(|s| s.get(selected.get()).map(|st| (st.view)()));
    let panel = move || {
        stories.with_value(|s| {
            s.get(selected.get()).map(|st| {
                let description = st.description();
                let args = st.args().to_vec();
                let source = st.source();
                view! {
                    <aside class="theoria-panel">
                        {description.map(|d| view! { <Markdown text=d /> })}
                        <Controls args=args />
                        {source.map(|src| view! { <ShowCode source=src /> })}
                    </aside>
                }
            })
        })
    };

    view! {
        <div class="theoria">
            <StoryNav names=names selected=selected set_selected=set_selected />
            <main class="theoria-stage">{stage}</main>
            {panel}
        </div>
    }
}

#[cfg(feature = "csr")]
fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// The persisted selected-story name, if any. `None` off the browser (ssr/tests).
fn load_selected() -> Option<String> {
    #[cfg(feature = "csr")]
    {
        storage()?.get_item(STORAGE_KEY).ok().flatten()
    }
    #[cfg(not(feature = "csr"))]
    {
        None
    }
}

/// Persist the selected-story name (no-op off the browser).
fn save_selected(name: &str) {
    #[cfg(feature = "csr")]
    {
        if let Some(s) = storage() {
            let _ = s.set_item(STORAGE_KEY, name);
        }
    }
    #[cfg(not(feature = "csr"))]
    {
        let _ = name;
    }
}
