//! dokime component tests for `AreaInspector`.

use leptos::prelude::*;
use slp_core::{Corner, ItemStatus};

use super::AreaInspector;

fn noop_f64() -> Callback<f64> {
    Callback::new(|_| {})
}

fn noop() -> Callback<()> {
    Callback::new(|()| {})
}

#[test]
fn shows_a_mulch_bed_with_depth_area_and_cost() {
    let html = dokime::render(move || {
        view! {
            <AreaInspector
                title="Mulch".to_string()
                category=Some("mulch-bed".to_string())
                area_ft2=80.0
                elevation=0.0
                depth=3.0
                show_depth=true
                cost=Some(29.63)
                corner=Corner::Ne
                on_elevation=noop_f64()
                on_depth=noop_f64()
                on_delete=noop()
            />
        }
    });
    assert!(html.contains("Mulch"), "the material name titles the panel");
    assert!(html.contains("mulch-bed"), "the material category");
    assert!(html.contains("80 ft²"), "the area readout");
    assert!(html.contains("$29.63"), "the cost readout");
    // A volume-priced material shows the depth field.
    assert!(html.contains(r#"data-testid="area-inspector-depth""#));
    assert!(html.contains(r#"data-testid="area-inspector-elevation""#));
    assert!(
        html.contains(r#"data-testid="delete-area""#),
        "a remove button"
    );
}

#[test]
fn a_per_area_material_hides_the_depth_field() {
    // A paver area (per-ft²) passes no depth — the depth field is absent, but
    // elevation stays editable.
    let html = dokime::render(move || {
        view! {
            <AreaInspector
                title="Pavers".to_string()
                category=Some("paver".to_string())
                area_ft2=80.0
                elevation=0.0
                depth=0.0
                show_depth=false
                cost=Some(640.0)
                corner=Corner::Nw
                on_elevation=noop_f64()
                on_depth=noop_f64()
                on_delete=noop()
            />
        }
    });
    assert!(html.contains("Pavers"));
    assert!(html.contains("$640.00"));
    assert_eq!(
        dokime::count(&html, r#"data-testid="area-inspector-depth""#),
        0,
        "no depth field for a per-ft² material"
    );
    assert!(html.contains(r#"data-testid="area-inspector-elevation""#));
}

#[test]
fn an_unpriced_area_shows_a_dash_for_cost() {
    let html = dokime::render(move || {
        view! {
            <AreaInspector
                title="Area".to_string()
                area_ft2=50.0
                elevation=0.0
                depth=0.0
                cost=None
                corner=Corner::Se
                on_elevation=noop_f64()
                on_depth=noop_f64()
                on_delete=noop()
            />
        }
    });
    assert!(html.contains("50 ft²"));
    assert!(html.contains("—"), "cost dashes out when unpriced");
}

#[test]
fn a_deck_level_shows_status_and_elevation_but_no_material_or_cost() {
    let html = dokime::render(move || {
        view! {
            <AreaInspector
                title="Deck".to_string()
                area_ft2=200.0
                elevation=2.0
                depth=0.0
                status=Some(ItemStatus::existing)
                corner=Corner::Ne
                on_elevation=noop_f64()
                on_depth=noop_f64()
                on_delete=noop()
            />
        }
    });
    assert!(html.contains("Deck"));
    assert!(html.contains("200 ft²"));
    // Structure mode: existing/planned buttons, no material or cost row.
    assert!(
        html.contains(r#"data-testid="area-status""#),
        "status control"
    );
    assert!(html.contains(r#"data-testid="area-status-existing""#));
    assert!(
        html.contains(r#"data-testid="area-inspector-elevation""#),
        "deck elevation"
    );
    assert_eq!(
        dokime::count(&html, r#"data-testid="area-inspector-cost""#),
        0,
        "a structure has no cost row"
    );
    assert!(
        !html.contains("Material"),
        "no material row for a structure"
    );
}

#[test]
fn a_house_shows_status_but_hides_elevation() {
    // The house sits at grade — its inspector has a status control but no
    // elevation field.
    let html = dokime::render(move || {
        view! {
            <AreaInspector
                title="House".to_string()
                area_ft2=1200.0
                elevation=0.0
                show_elevation=false
                depth=0.0
                status=Some(ItemStatus::planned)
                corner=Corner::Nw
                on_elevation=noop_f64()
                on_depth=noop_f64()
                on_delete=noop()
            />
        }
    });
    assert!(html.contains("House"));
    assert!(html.contains("1200 ft²"));
    assert!(html.contains(r#"data-testid="area-status""#));
    assert_eq!(
        dokime::count(&html, r#"data-testid="area-inspector-elevation""#),
        0,
        "the house hides the elevation field"
    );
}

#[test]
fn the_corner_is_exposed_for_positioning() {
    let html = dokime::render(move || {
        view! {
            <AreaInspector
                title="Mulch".to_string()
                area_ft2=1.0
                elevation=0.0
                depth=0.0
                corner=Corner::Sw
                on_elevation=noop_f64()
                on_depth=noop_f64()
                on_delete=noop()
            />
        }
    });
    assert!(html.contains(r#"data-corner="sw""#));
}
