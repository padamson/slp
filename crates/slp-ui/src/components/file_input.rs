//! A file picker that reads the chosen file to a `data:` URI and hands it back.
//! Used to attach a material photo in the catalog editor. The read is a browser
//! (`csr`) concern — `web_sys::FileReader` — so it's gated; under SSR (dokime)
//! the `<input>` still renders, it just doesn't wire the read.

use leptos::prelude::*;

#[component]
pub fn FileInput(
    label: &'static str,
    testid: &'static str,
    /// The `accept` attribute (e.g. `image/*`).
    #[prop(default = "*/*")]
    accept: &'static str,
    /// Called with the picked file's contents as a `data:` URI.
    on_file: Callback<String>,
) -> impl IntoView {
    view! {
        <label class="file-input">
            {label}
            " "
            <input
                type="file"
                data-testid=testid
                accept=accept
                on:change=move |ev| read_data_url(&ev, on_file)
            />
        </label>
    }
}

/// Read the file just picked in `ev`'s `<input>` to a `data:` URI and hand it to
/// `on_file`. Browser-only — a no-op when not compiled for `csr`.
#[cfg(feature = "csr")]
fn read_data_url(ev: &leptos::ev::Event, on_file: Callback<String>) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;
    use web_sys::{FileReader, HtmlInputElement};

    let Some(file) = ev
        .target()
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .and_then(|input| input.files())
        .and_then(|list| list.get(0))
    else {
        return;
    };
    let Ok(reader) = FileReader::new() else {
        return;
    };
    let reader_for_load = reader.clone();
    // One-shot onload: when the read finishes, forward the data URI. `forget`
    // leaks the closure so it outlives this call and survives until it fires.
    let onload = Closure::<dyn FnMut()>::new(move || {
        if let Some(url) = reader_for_load.result().ok().and_then(|v| v.as_string()) {
            on_file.run(url);
        }
    });
    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();
    let _ = reader.read_as_data_url(&file);
}

#[cfg(not(feature = "csr"))]
fn read_data_url(_ev: &leptos::ev::Event, _on_file: Callback<String>) {}
