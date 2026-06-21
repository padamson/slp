//! The sidebar: one button per story name; the selected one is marked `active`.
//! A leaf component (no children) — easy to unit-test and to use as an e2e
//! fixture.

use leptos::prelude::*;

#[component]
pub fn StoryNav(
    names: Vec<&'static str>,
    selected: ReadSignal<usize>,
    set_selected: WriteSignal<usize>,
) -> impl IntoView {
    let items: Vec<_> = names
        .into_iter()
        .enumerate()
        .map(|(i, name)| {
            view! {
                <li>
                    <button
                        class:active=move || selected.get() == i
                        on:click=move |_| set_selected.set(i)
                    >
                        {name}
                    </button>
                </li>
            }
        })
        .collect();

    view! {
        <nav class="theoria-nav">
            <ul>{items}</ul>
        </nav>
    }
}
