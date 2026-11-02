//! leptotes component test for `ShowCode`.

use leptos::prelude::*;

use super::ShowCode;

#[test]
fn renders_the_source_in_a_code_block() {
    let html = leptotes::render(|| {
        view! { <ShowCode source="view! { <Yard /> }" /> }
    });
    assert!(html.contains("theoria-code"), "tagged for queries");
    assert!(html.contains("Show code"), "the toggle label");
    assert!(
        html.contains("&lt;Yard /&gt;") || html.contains("<Yard />"),
        "renders the source"
    );
}
