//! A labeled checkbox toggle. Controlled — the parent owns the checked state and
//! handles the change.

use leptos::prelude::*;

#[component]
pub fn Toggle(
    label: &'static str,
    testid: &'static str,
    #[prop(into)] checked: Signal<bool>,
    on_toggle: Callback<bool>,
) -> impl IntoView {
    view! {
        <label class="toggle">
            <input
                type="checkbox"
                data-testid=testid
                prop:checked=move || checked.get()
                on:change=move |ev| on_toggle.run(event_target_checked(&ev))
            />
            " "
            {label}
        </label>
    }
}
