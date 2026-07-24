//! dokime component tests for `IngestDraft` (the curation step).

use leptos::prelude::*;
use slp_core::CatalogItem;

use super::IngestDraft;
use crate::vision::{ExtractedProduct, PriceUnitHint, SizeVariant, Variant};

fn color(name: &str, available: bool) -> Variant {
    Variant {
        name: name.to_string(),
        available,
        bbox: None,
        swatch: None,
    }
}

fn size(name: &str, w: f64, d: f64) -> SizeVariant {
    SizeVariant {
        name: name.to_string(),
        available: true,
        width_ft: Some(w),
        depth_ft: Some(d),
        thickness_in: Some(2.375),
        includes: None,
    }
}

fn sample() -> ExtractedProduct {
    ExtractedProduct {
        name: "Blu 60 Slate Slabs".to_string(),
        category: Some("slab".to_string()),
        price_unit: Some(PriceUnitHint::PerSquareFoot),
        unit_price: None,
        colors: vec![color("Shale Grey", true), color("Onyx Black", false)],
        textures: vec![color("Slate", true)],
        sizes: vec![
            SizeVariant {
                name: "60 MM".to_string(),
                available: true,
                width_ft: Some(1.083),
                depth_ft: Some(1.083),
                thickness_in: Some(2.375),
                includes: Some("A: 6½×13, B: 13×13, C: 19½×13 in".to_string()),
            },
            size("Grande", 2.71, 1.63),
        ],
        patterns: Vec::new(),
        notes: Some("No price listed.".to_string()),
    }
}

fn render(product: ExtractedProduct) -> String {
    dokime::render(move || {
        let product = product.clone();
        view! {
            <IngestDraft
                product=product
                on_add=Callback::new(|_: Vec<CatalogItem>| {})
                on_discard=Callback::new(|()| {})
            />
        }
    })
}

#[test]
fn offers_a_checkbox_per_color_and_size_with_dimensions() {
    let html = render(sample());
    assert!(
        html.contains(r#"data-testid="ingest-color-0""#),
        "a color checkbox"
    );
    assert!(html.contains(r#"data-testid="ingest-color-1""#));
    assert!(
        html.contains(r#"data-testid="ingest-size-0""#),
        "a size checkbox"
    );
    assert!(html.contains("Shale Grey"));
    assert!(html.contains("Onyx Black"));
    // Sizes show their dimensions.
    assert!(html.contains("1.08×1.08 ft"), "the size's dimensions");
    // Editable shared fields (category / price / price basis) + the notes.
    assert!(html.contains(r#"data-testid="ingest-draft-category""#));
    assert!(html.contains(r#"data-testid="ingest-draft-price""#));
    assert!(
        html.contains(r#"data-testid="ingest-draft-price-unit""#),
        "the price basis is editable"
    );
    assert!(html.contains("No price listed."), "notes surfaced");
    // A multi-piece format shows its included pieces as metadata.
    assert!(
        html.contains("incl. A: 6½×13"),
        "the included pieces are surfaced: {html}"
    );
}

#[test]
fn a_color_with_a_cropped_swatch_shows_a_thumbnail() {
    let mut p = sample();
    p.colors[0].swatch = Some("data:image/png;base64,SWATCH".to_string());
    let html = render(p);
    assert!(
        html.contains(r#"data-testid="ingest-color-swatch-0""#),
        "the cropped swatch shows as a thumbnail"
    );
    assert!(html.contains("data:image/png;base64,SWATCH"));
    // A color without a swatch has no thumbnail.
    assert_eq!(
        dokime::count(&render(sample()), r#"data-testid="ingest-color-swatch-0""#),
        0,
        "no thumbnail before cropping"
    );
}

#[test]
fn unavailable_options_are_dimmed_and_disabled() {
    let html = render(sample());
    // Onyx Black is unavailable.
    assert!(
        html.contains("unavailable"),
        "an unavailable option is dimmed"
    );
    assert!(html.contains("disabled"), "and its checkbox is disabled");
}

#[test]
fn the_approve_button_counts_the_selected_combos() {
    // 2 colors (one unavailable → 1 ticked) × 2 sizes ticked = 2 items.
    let html = render(sample());
    assert!(
        html.contains(r#"data-testid="ingest-approve""#),
        "an approve button"
    );
    assert!(
        html.contains("Add 2 to catalog"),
        "the count is available colors × available sizes: {html}"
    );
    assert!(
        html.contains(r#"data-testid="ingest-discard""#),
        "a discard action"
    );
}

#[test]
fn laying_patterns_list_as_ticked_checkboxes_with_diagram_thumbnails() {
    // A draft with patterns shows a "Laying patterns" group: every pattern
    // starts ticked (they ride items, they don't multiply them — the count
    // stays colors × sizes), and one with a cropped diagram shows it as a
    // clickable thumbnail; one without shows no thumbnail.
    let mut p = sample();
    p.patterns = vec![
        crate::vision::Pattern {
            name: "Herringbone".to_string(),
            bbox: None,
            diagram: Some("data:image/png;base64,DIAG".to_string()),
        },
        crate::vision::Pattern {
            name: "Linear".to_string(),
            bbox: None,
            diagram: None,
        },
    ];
    let html = render(p);
    assert!(html.contains("Laying patterns"), "the group renders");
    assert!(
        html.contains(r#"data-testid="ingest-pattern-0""#)
            && html.contains(r#"data-testid="ingest-pattern-1""#),
        "a checkbox per pattern"
    );
    assert!(
        html.contains(r#"data-testid="ingest-pattern-diagram-0""#),
        "a diagram thumbnail for the cropped pattern"
    );
    assert!(html.contains("data:image/png;base64,DIAG"));
    assert_eq!(
        dokime::count(&html, r#"data-testid="ingest-pattern-diagram-1""#),
        0,
        "no thumbnail without a diagram"
    );
    assert!(
        html.contains("Add 2 to catalog"),
        "patterns don't multiply the item count"
    );
}

#[test]
fn a_draft_without_patterns_shows_no_patterns_group() {
    let html = render(sample());
    assert!(!html.contains("Laying patterns"));
    assert_eq!(dokime::count(&html, r#"data-testid="ingest-pattern-0""#), 0);
}

#[test]
fn a_slab_with_no_extracted_price_basis_defaults_to_per_square_foot() {
    // Manufacturer pages usually show no price basis; a slab/paver must still
    // land per-ft² (a per-item fallback would hide it from the Area picker).
    let mut p = sample();
    p.price_unit = None;
    let html = render(p);
    assert!(
        html.contains(r#"value="per_square_foot" selected"#),
        "the category defaults the price basis: {html}"
    );
}
