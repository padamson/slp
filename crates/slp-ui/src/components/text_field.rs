//! A labeled text input. Controlled — the parent owns the value; every
//! keystroke is reported via `on_input`. The string counterpart of
//! [`NumberField`](super::NumberField), used to edit catalog metadata like an
//! item's name or category.

use leptos::prelude::*;

#[component]
pub fn TextField(
    label: &'static str,
    testid: &'static str,
    #[prop(into)] value: Signal<String>,
    on_input: Callback<String>,
    /// Placeholder shown when the field is empty.
    #[prop(default = "", into)]
    placeholder: &'static str,
    /// The HTML input type — `"text"` by default, `"password"` to mask a secret
    /// (e.g. an API key) so it isn't shown on screen.
    #[prop(default = "text")]
    input_type: &'static str,
) -> impl IntoView {
    view! {
        <label class="text-field">
            {label}
            " "
            <input
                type=input_type
                data-testid=testid
                placeholder=placeholder
                // `value` renders the current text server-side; `prop:value`
                // keeps the live control in sync on the client.
                value=move || value.get()
                prop:value=move || value.get()
                on:input=move |ev| on_input.run(event_target_value(&ev))
            />
        </label>
    }
}
