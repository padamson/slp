//! A drawing-tool toggle button: a label that highlights when its tool is the
//! active one. Controlled — the parent owns the active state and the pick event.

use leptos::prelude::*;

#[component]
pub fn ToolButton(
    label: &'static str,
    testid: &'static str,
    #[prop(into)] active: Signal<bool>,
    on_pick: Callback<()>,
) -> impl IntoView {
    view! {
        <button
            data-testid=testid
            class:active=move || active.get()
            on:click=move |_| on_pick.run(())
        >
            {label}
        </button>
    }
}
