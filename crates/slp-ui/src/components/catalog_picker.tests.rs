//! dokime component tests for `CatalogPicker`.

use leptos::prelude::*;
use slp_core::CatalogItem;

use super::CatalogPicker;

fn item(id: &str, name: &str) -> CatalogItem {
    let mut c = CatalogItem::new(id.to_string());
    c.name = Some(name.to_string());
    c
}

#[test]
fn renders_an_option_per_catalog_item() {
    let html = dokime::render(|| {
        view! {
            <CatalogPicker
                testid="picker"
                catalog=Signal::derive(|| vec![item("sofa", "Sofa"), item("chair", "Armchair")])
                selected=Signal::derive(|| "sofa".to_string())
                on_pick=Callback::new(|_| {})
            />
        }
    });
    assert_eq!(
        dokime::count(&html, "<option"),
        2,
        "one option per catalog item"
    );
    assert!(
        html.contains(">Sofa</option>") && html.contains(">Armchair</option>"),
        "each item's name labels its option"
    );
    assert!(
        html.contains(r#"data-testid="picker""#),
        "tagged for queries"
    );
}

#[test]
fn falls_back_to_the_id_when_an_item_has_no_name() {
    let html = dokime::render(|| {
        view! {
            <CatalogPicker
                testid="picker"
                catalog=Signal::derive(|| vec![CatalogItem::new("mystery".to_string())])
                selected=Signal::derive(String::new)
                on_pick=Callback::new(|_| {})
            />
        }
    });
    assert!(
        html.contains(">mystery</option>"),
        "the id is the fallback label"
    );
}

#[test]
fn empty_catalog_has_no_options() {
    let html = dokime::render(|| {
        view! {
            <CatalogPicker
                testid="picker"
                catalog=Signal::derive(Vec::new)
                selected=Signal::derive(String::new)
                on_pick=Callback::new(|_| {})
            />
        }
    });
    assert_eq!(dokime::count(&html, "<option"), 0);
}
