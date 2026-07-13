//! A drawing-tool toggle button: a label that highlights when its tool is the
//! active one. Controlled — the parent owns the active state and the pick event.
//! An optional leading slot (`children`) renders before the label — used by the
//! material picker to show a swatch beside the material name. It can also be
//! rendered `disabled` (with an optional `title` tooltip) for a control that
//! isn't available in the current browser.

use leptos::prelude::*;

#[component]
pub fn ToolButton(
    label: &'static str,
    testid: &'static str,
    #[prop(into)] active: Signal<bool>,
    on_pick: Callback<()>,
    /// Greys the button out and blocks clicks (a browser won't fire `click` on
    /// a disabled `<button>`) — for a control unavailable in this browser.
    #[prop(default = false)]
    disabled: bool,
    /// A hover tooltip (native `title`) — e.g. why a disabled control is off.
    #[prop(default = "")]
    title: &'static str,
    /// Optional leading content rendered before the label (e.g. a swatch).
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    view! {
        <button
            data-testid=testid
            class:active=move || active.get()
            disabled=disabled
            title=title
            on:click=move |_| on_pick.run(())
        >
            {children.map(|c| c())}
            {label}
        </button>
    }
}
