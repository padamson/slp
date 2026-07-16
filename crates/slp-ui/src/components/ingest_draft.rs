//! The screenshot-ingestion curation step (M4.2): review an extracted draft
//! product and **multi-select** which color × size combinations to keep. Each
//! ticked combo becomes one catalog item (a color's look at a size's
//! dimensions), sharing the edited category / price. Nothing is added until the
//! user approves — a misread never silently changes the plan.

use leptos::prelude::*;
use slp_core::{CatalogItem, PriceUnit};

use super::catalog_panel::{price_unit_from_id, price_unit_id, price_unit_options};
use super::{NumberField, SelectField, TextField};
use crate::vision::{ExtractedProduct, to_catalog_items};

/// One size-format row's rendering metadata: `(label, available, width_ft,
/// depth_ft, thickness_in, includes)`.
type SizeRow = (
    String,
    bool,
    Option<f64>,
    Option<f64>,
    Option<f64>,
    Option<String>,
);

#[allow(clippy::too_many_lines)]
#[component]
pub fn IngestDraft(
    /// The extracted draft to curate.
    product: ExtractedProduct,
    /// Approve: add these catalog items (one per selected combo).
    on_add: Callback<Vec<CatalogItem>>,
    /// Discard the draft without adding anything.
    #[prop(default = Callback::new(|(): ()| {}))]
    on_discard: Callback<()>,
) -> impl IntoView {
    // Rendering metadata (owned, so the reactive closures don't borrow `product`).
    let name = product.name.clone();
    let notes = product.notes.clone();
    let colors: Vec<(String, bool)> = product
        .colors
        .iter()
        .map(|c| (c.name.clone(), c.available))
        .collect();
    let sizes: Vec<SizeRow> = product
        .sizes
        .iter()
        .map(|s| {
            (
                s.name.clone(),
                s.available,
                s.width_ft,
                s.depth_ft,
                s.thickness_in,
                s.includes.clone(),
            )
        })
        .collect();
    let has_colors = !colors.is_empty();
    let has_sizes = !sizes.is_empty();
    // The extracted price basis, editable here before adding.
    let init_pu: PriceUnit = product.price_unit.map_or(
        PriceUnit::per_item,
        super::super::vision::PriceUnitHint::to_price_unit,
    );

    // Editable state, seeded from the draft; available options start ticked.
    let color_checks = RwSignal::new(colors.iter().map(|(_, a)| *a).collect::<Vec<bool>>());
    let size_checks = RwSignal::new(
        sizes
            .iter()
            .map(|(_, a, _, _, _, _)| *a)
            .collect::<Vec<bool>>(),
    );
    let category = RwSignal::new(product.category.clone().unwrap_or_default());
    let price = RwSignal::new(0.0_f64);
    let price_unit = RwSignal::new(init_pu);
    // The product itself, for building items on approve.
    let product = StoredValue::new(product);

    // Selected indices, and the resulting item count (an axis with no options is
    // a single implicit unit; an axis with options but none ticked is zero).
    let selected = move || {
        let cols: Vec<usize> = color_checks
            .get()
            .iter()
            .enumerate()
            .filter_map(|(i, &b)| b.then_some(i))
            .collect();
        let szs: Vec<usize> = size_checks
            .get()
            .iter()
            .enumerate()
            .filter_map(|(i, &b)| b.then_some(i))
            .collect();
        (cols, szs)
    };
    let count = move || {
        let (cols, szs) = selected();
        let c = if has_colors { cols.len() } else { 1 };
        let s = if has_sizes { szs.len() } else { 1 };
        c * s
    };

    let approve = move |_| {
        let (cols, szs) = selected();
        if (has_colors && cols.is_empty()) || (has_sizes && szs.is_empty()) {
            return;
        }
        let unit_price = (price.get() > 0.0).then(|| price.get());
        let pu = price_unit.get();
        let items = product
            .with_value(|p| to_catalog_items(p, &cols, &szs, &category.get(), unit_price, &pu));
        on_add.run(items);
    };

    let color_rows = colors
        .into_iter()
        .enumerate()
        .map(|(i, (label, avail))| {
            view! {
                <label class="ingest-check" class:unavailable=!avail>
                    <input
                        type="checkbox"
                        data-testid=format!("ingest-color-{i}")
                        prop:checked=move || color_checks.get().get(i).copied().unwrap_or(false)
                        disabled=!avail
                        on:change=move |_| {
                            color_checks
                                .update(|v| {
                                    if let Some(b) = v.get_mut(i) {
                                        *b = !*b;
                                    }
                                });
                        }
                    />
                    {label}
                </label>
            }
        })
        .collect::<Vec<_>>();

    let size_rows = sizes
        .into_iter()
        .enumerate()
        .map(|(i, (label, avail, w, d, t, incl))| {
            let dims = match (w, d, t) {
                (Some(w), Some(d), Some(t)) => format!(" ({w:.2}×{d:.2} ft · {t:.1} in)"),
                (Some(w), Some(d), None) => format!(" ({w:.2}×{d:.2} ft)"),
                (_, _, Some(t)) => format!(" ({t:.1} in)"),
                _ => String::new(),
            };
            view! {
                <label class="ingest-check" class:unavailable=!avail>
                    <input
                        type="checkbox"
                        data-testid=format!("ingest-size-{i}")
                        prop:checked=move || size_checks.get().get(i).copied().unwrap_or(false)
                        disabled=!avail
                        on:change=move |_| {
                            size_checks
                                .update(|v| {
                                    if let Some(b) = v.get_mut(i) {
                                        *b = !*b;
                                    }
                                });
                        }
                    />
                    <span>
                        {format!("{label}{dims}")}
                        {incl
                            .map(|inc| {
                                view! { <span class="ingest-includes">{format!(" · incl. {inc}")}</span> }
                            })}
                    </span>
                </label>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <div class="ingest-draft" data-testid="ingest-draft">
            <h4 class="ingest-draft-name">{name}</h4>
            {notes.map(|n| view! { <p class="ingest-draft-notes">{n}</p> })}
            <TextField
                label="Category"
                testid="ingest-draft-category"
                value=category
                on_input=Callback::new(move |v: String| category.set(v))
            />
            <NumberField
                label="Price ($)"
                testid="ingest-draft-price"
                value=Signal::derive(move || price.get())
                step=1.0
                min=0.0
                on_input=Callback::new(move |v: f64| price.set(v))
            />
            <SelectField
                label="Priced"
                testid="ingest-draft-price-unit"
                value=Signal::derive(move || price_unit_id(&price_unit.get()).to_string())
                options=price_unit_options()
                on_change=Callback::new(move |id: String| price_unit.set(price_unit_from_id(&id)))
            />
            {(!color_rows.is_empty())
                .then(|| {
                    view! {
                        <div class="ingest-draft-group">
                            <span class="ingest-draft-label">"Colors"</span>
                            <div class="ingest-checks">{color_rows}</div>
                        </div>
                    }
                })}
            {(!size_rows.is_empty())
                .then(|| {
                    view! {
                        <div class="ingest-draft-group">
                            <span class="ingest-draft-label">"Sizes"</span>
                            <div class="ingest-checks">{size_rows}</div>
                        </div>
                    }
                })}
            <div class="ingest-draft-actions">
                <button
                    class="ingest-approve"
                    data-testid="ingest-approve"
                    disabled=move || count() == 0
                    on:click=approve
                >
                    {move || format!("Add {} to catalog", count())}
                </button>
                <button
                    class="ingest-discard"
                    data-testid="ingest-discard"
                    on:click=move |_| on_discard.run(())
                >
                    "Discard"
                </button>
            </div>
        </div>
    }
}
