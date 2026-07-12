//! dokime component tests for `MaterialSwatch`.

use leptos::prelude::*;

use super::MaterialSwatch;
use crate::style::{MULCH_FILL, PAVER_FILL, SHAPE_FILL};

#[test]
fn a_material_with_an_image_shows_a_photo_thumbnail() {
    let html = dokime::render(|| {
        view! {
            <MaterialSwatch
                image=Some("data:image/png;base64,AAAA".to_string())
                category=Some("paver".to_string())
            />
        }
    });
    assert!(html.contains("<img"), "a photo thumbnail: {html}");
    assert!(
        html.contains(r#"src="data:image/png;base64,AAAA""#),
        "shows the material image"
    );
    assert!(
        !html.contains("background-color"),
        "no flat-color square when there's a photo"
    );
}

#[test]
fn a_material_without_an_image_shows_its_category_color() {
    // Paver → gray, mulch → brown, uncategorized → the neutral default.
    let paver = dokime::render(|| view! { <MaterialSwatch category=Some("paver".to_string()) /> });
    assert!(
        paver.contains(&format!("background-color: {PAVER_FILL}")),
        "paver swatch is gray: {paver}"
    );
    assert!(!paver.contains("<img"), "no photo when there's no image");

    let mulch =
        dokime::render(|| view! { <MaterialSwatch category=Some("mulch-bed".to_string()) /> });
    assert!(
        mulch.contains(&format!("background-color: {MULCH_FILL}")),
        "mulch swatch is brown: {mulch}"
    );

    let plain = dokime::render(|| view! { <MaterialSwatch /> });
    assert!(
        plain.contains(&format!("background-color: {SHAPE_FILL}")),
        "an uncategorized swatch is the neutral default: {plain}"
    );
}

#[test]
fn an_empty_image_string_falls_back_to_the_color() {
    // A material whose image was cleared to "" reads as no-photo, not a broken
    // <img src="">.
    let html = dokime::render(|| {
        view! { <MaterialSwatch image=Some(String::new()) category=Some("paver".to_string()) /> }
    });
    assert!(!html.contains("<img"), "an empty image is not a photo");
    assert!(
        html.contains(&format!("background-color: {PAVER_FILL}")),
        "falls back to the category color"
    );
}
