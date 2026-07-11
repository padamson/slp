//! dokime component test for `FileInput`. (The file-read itself is browser-only
//! — csr — so it's exercised in the e2e suite, not here.)

use leptos::prelude::*;

use super::FileInput;

#[test]
fn renders_a_file_input_with_accept_and_testid() {
    let html = dokime::render(|| {
        view! {
            <FileInput
                label="Upload"
                testid="catalog-image-file"
                accept="image/*"
                on_file=Callback::new(|_| {})
            />
        }
    });
    assert!(html.contains("Upload"), "label");
    assert!(html.contains(r#"type="file""#), "a file input");
    assert!(
        html.contains(r#"data-testid="catalog-image-file""#),
        "testid"
    );
    assert!(html.contains(r#"accept="image/*""#), "the accept filter");
}
