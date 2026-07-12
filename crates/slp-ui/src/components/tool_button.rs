//! A drawing-tool toggle button: a label that highlights when its tool is the
//! active one. Controlled — the parent owns the active state and the pick event.
//! An optional leading slot (`children`) renders before the label — used by the
//! material picker to show a swatch beside the material name.

use leptos::prelude::*;

#[component]
pub fn ToolButton(
    label: &'static str,
    testid: &'static str,
    #[prop(into)] active: Signal<bool>,
    on_pick: Callback<()>,
    /// Optional leading content rendered before the label (e.g. a swatch).
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    view! {
        <button
            data-testid=testid
            class:active=move || active.get()
            on:click=move |_| on_pick.run(())
        >
            {children.map(|c| c())}
            {label}
        </button>
    }
}
