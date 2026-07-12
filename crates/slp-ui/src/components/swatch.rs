//! A small material swatch: the material's photo thumbnail when it has one,
//! else a flat square in its category color. It's the panel/inspector
//! counterpart of how a drawn area fills on the canvas (a tiled photo, or the
//! flat category color when there's none) — used in the catalog list, the area
//! material picker, and the area inspector so a material reads the same
//! everywhere.

use leptos::prelude::*;

use crate::style::area_style;

// `category` is an owned prop (Leptos components take owned props) that this
// only reads by reference — not worth a `&str` + lifetime.
#[allow(clippy::needless_pass_by_value)]
#[component]
pub fn MaterialSwatch(
    /// The material's image (a `data:` URI or URL): a photo thumbnail when set
    /// and non-empty, otherwise the flat category color.
    #[prop(default = None)]
    image: Option<String>,
    /// The material's category (`paver`, `mulch-bed`, …), which picks the flat
    /// fallback color when there's no image.
    #[prop(default = None)]
    category: Option<String>,
) -> impl IntoView {
    if let Some(src) = image.filter(|s| !s.is_empty()) {
        view! {
            <img class="material-swatch" data-testid="material-swatch" src=src alt="" />
        }
        .into_any()
    } else {
        // Same category → color mapping the canvas fill uses, so the swatch
        // matches the drawn area.
        let (fill, _) = area_style(category.as_deref());
        view! {
            <span
                class="material-swatch material-swatch--color"
                data-testid="material-swatch"
                style=format!("background-color: {fill}")
            />
        }
        .into_any()
    }
}
