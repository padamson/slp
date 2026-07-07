//! A labeled dropdown. Controlled — the parent owns the selected value; picking
//! an option reports its value via `on_change`. The choice-from-a-fixed-set
//! counterpart of [`NumberField`](super::NumberField)/[`TextField`](super::TextField),
//! used to pick a material's `price_unit` or (later) a course's material.

use leptos::prelude::*;

#[component]
pub fn SelectField(
    label: &'static str,
    testid: &'static str,
    #[prop(into)] value: Signal<String>,
    /// Selectable options as `(value, label)` pairs, in display order.
    options: Vec<(String, String)>,
    on_change: Callback<String>,
) -> impl IntoView {
    view! {
        <label class="select-field">
            {label}
            " "
            <select
                data-testid=testid
                prop:value=move || value.get()
                on:change=move |ev| on_change.run(event_target_value(&ev))
            >
                {options
                    .into_iter()
                    .map(|(val, lbl)| {
                        // Marked server-side via `selected`; `prop:value` on the
                        // <select> keeps the live control in sync on the client.
                        let this = val.clone();
                        view! {
                            <option value=val selected=move || value.get() == this>
                                {lbl}
                            </option>
                        }
                    })
                    .collect::<Vec<_>>()}
            </select>
        </label>
    }
}
