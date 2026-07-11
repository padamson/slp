//! M4.3 e2e: open the catalog inspector, edit a catalog item's price and
//! footprint, and confirm the edits propagate live to every object placed from
//! it — the estimate reprices and the footprint re-renders, because an object
//! references its catalog item by `catalog_ref` (not a copy).
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use common::{dist_dir, measure_ppf, place, serve};
use playwright_rs::protocol::Playwright;
use playwright_rs::{Locator, expect};

/// Poll a locator's `text_content` until it equals `want` or times out.
async fn wait_text(loc: &Locator, want: &str) -> Result<()> {
    let start = Instant::now();
    loop {
        let text = loc.text_content().await?.unwrap_or_default();
        if text.trim() == want {
            return Ok(());
        }
        if start.elapsed() >= Duration::from_secs(5) {
            return Err(anyhow!("expected '{want}', last was '{text}'"));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// Poll a locator's numeric attribute until it satisfies `pred` or times out.
async fn wait_attr_f64(loc: &Locator, attr: &str, pred: impl Fn(f64) -> bool) -> Result<f64> {
    let start = Instant::now();
    loop {
        if let Some(v) = loc.get_attribute(attr).await?.and_then(|s| s.parse().ok())
            && pred(v)
        {
            return Ok(v);
        }
        if start.elapsed() >= Duration::from_secs(5) {
            return Err(anyhow!("attribute '{attr}' never satisfied the predicate"));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test]
async fn edits_a_catalog_item_and_propagates_to_placed_objects() -> Result<()> {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!(
            "skipping: {} not built (run `trunk build`).",
            dist.display()
        );
        return Ok(());
    }

    let (addr, _server) = serve(&dist).await?;
    let pw = Playwright::launch().await.context("launch playwright")?;
    let browser = pw.chromium().launch().await.context("launch chromium")?;
    let page = browser.new_page().await.context("new page")?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;

    let yard = page.locator("[data-testid='yard']").await;
    let ppf = measure_ppf(&yard).await?;

    // Place one lounge chair (the default first catalog item) — the only priced
    // object on the plan, so the grand total is just its price.
    place(&page, &yard, ppf, 35.0, 15.0).await?;
    let footprint = page.locator("[data-testid='yard'] .furniture-item rect").await;
    let start_w = footprint
        .get_attribute("width")
        .await?
        .and_then(|s| s.parse::<f64>().ok())
        .context("the chair footprint has a width")?;

    // Open the catalog inspector and select the lounge chair.
    page.locator("[data-testid='edit-catalog']")
        .await
        .click(None)
        .await
        .context("open the catalog inspector")?;
    page.locator("[data-testid='catalog-row-lounge-chair']")
        .await
        .click(None)
        .await
        .context("select the lounge chair")?;
    expect(page.locator("[data-testid='catalog-editor']").await)
        .to_be_visible()
        .await
        .context("the editor opens for the selected item")?;

    // Reprice it to $500 — the estimate's grand total follows immediately.
    page.locator("[data-testid='catalog-price']")
        .await
        .fill("500", None)
        .await
        .context("set the chair's price to $500")?;
    wait_text(&page.locator("[data-testid='estimate-total']").await, "$500.00")
        .await
        .context("the estimate reprices the placed chair live")?;

    // Widen its footprint — the placed chair's rendered rect grows (width in px
    // is width_ft × px_ft), proving the render follows the catalog too.
    page.locator("[data-testid='catalog-width']")
        .await
        .fill("8", None)
        .await
        .context("widen the chair to 8 ft")?;
    wait_attr_f64(&footprint, "width", |w| w > start_w + 1.0)
        .await
        .context("the placed chair's footprint grows to match the new width")?;

    // Close the panel.
    page.locator("[data-testid='catalog-close']")
        .await
        .click(None)
        .await
        .context("close the catalog inspector")?;
    expect(page.locator("[data-testid='catalog-panel']").await)
        .to_have_count(0)
        .await
        .context("the catalog inspector closes")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn sets_a_material_image_that_previews_and_persists() -> Result<()> {
    // M4.4: give a material a photo (a data: URI), see it previewed in the
    // editor, and confirm it survives a reload.
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!(
            "skipping: {} not built (run `trunk build`).",
            dist.display()
        );
        return Ok(());
    }

    let (addr, _server) = serve(&dist).await?;
    let pw = Playwright::launch().await.context("launch playwright")?;
    let browser = pw.chromium().launch().await.context("launch chromium")?;
    let page = browser.new_page().await.context("new page")?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;

    // A 1×1 transparent PNG, so it's self-contained (no network in the test).
    let img = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==";

    // Open the catalog inspector, edit the Pavers material, set its image.
    page.locator("[data-testid='edit-catalog']")
        .await
        .click(None)
        .await
        .context("open the catalog inspector")?;
    page.locator("[data-testid='catalog-row-paver']")
        .await
        .click(None)
        .await
        .context("select the paver material")?;
    page.locator("[data-testid='catalog-image']")
        .await
        .fill(img, None)
        .await
        .context("set the material image")?;

    // The editor previews it.
    let preview = page.locator("[data-testid='catalog-image-preview']").await;
    expect(preview.clone())
        .to_have_count(1)
        .await
        .context("the image previews in the editor")?;
    assert_eq!(
        preview.get_attribute("src").await?.as_deref(),
        Some(img),
        "the preview shows the set image"
    );

    // Reload — the image persists on the catalog item.
    page.reload(None).await.context("reload the page")?;
    page.locator("[data-testid='edit-catalog']")
        .await
        .click(None)
        .await
        .context("reopen the catalog inspector")?;
    page.locator("[data-testid='catalog-row-paver']")
        .await
        .click(None)
        .await
        .context("reselect the paver material")?;
    assert_eq!(
        page.locator("[data-testid='catalog-image']")
            .await
            .input_value(None)
            .await?,
        img,
        "the image persisted across a reload"
    );

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn adds_and_authors_a_material_that_persists() -> Result<()> {
    // B3.0: hand-add a catalog material, set its name/price/price_unit, and
    // confirm it survives a reload — the prerequisite for composing an area from
    // materials you added yourself.
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!(
            "skipping: {} not built (run `trunk build`).",
            dist.display()
        );
        return Ok(());
    }

    let (addr, _server) = serve(&dist).await?;
    let pw = Playwright::launch().await.context("launch playwright")?;
    let browser = pw.chromium().launch().await.context("launch chromium")?;
    let page = browser.new_page().await.context("new page")?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;

    // Open the catalog inspector and add a new material.
    page.locator("[data-testid='edit-catalog']")
        .await
        .click(None)
        .await
        .context("open the catalog inspector")?;
    page.locator("[data-testid='catalog-add']")
        .await
        .click(None)
        .await
        .context("add a new material")?;
    expect(page.locator("[data-testid='catalog-editor']").await)
        .to_be_visible()
        .await
        .context("the new item is selected for editing")?;

    // Author it: a per-yd³ river gravel at $60.
    page.locator("[data-testid='catalog-name']")
        .await
        .fill("River gravel", None)
        .await
        .context("name the material")?;
    page.locator("[data-testid='catalog-price']")
        .await
        .fill("60", None)
        .await
        .context("price the material")?;
    // A new material defaults to a bulk (per-yd³) unit — change it to per-ft² to
    // prove the control mutates it.
    let unit = page.locator("[data-testid='catalog-price-unit']").await;
    unit.select_option("per_square_foot", None)
        .await
        .context("price it per square foot")?;
    assert_eq!(
        unit.input_value(None).await?,
        "per_square_foot",
        "the price unit is set"
    );

    // A material is never a placeable object — it must not appear in the palette.
    expect(page.locator("[data-testid='palette-material-1']").await)
        .to_have_count(0)
        .await
        .context("the added material is catalog-only, not a placeable tile")?;

    // Reload — the authored material persists (catalog rides localStorage).
    page.reload(None).await.context("reload the page")?;
    page.locator("[data-testid='edit-catalog']")
        .await
        .click(None)
        .await
        .context("reopen the catalog inspector")?;
    let row = page.locator("[data-testid='catalog-row-material-1']").await;
    expect(row.clone())
        .to_have_count(1)
        .await
        .context("the added material survived the reload")?;
    let row_text = row.text_content().await?.unwrap_or_default();
    assert!(row_text.contains("River gravel"), "authored name persisted: {row_text}");
    assert!(row_text.contains("$60"), "authored price persisted: {row_text}");
    // Its price unit persisted too.
    row.click(None).await.context("reselect the material")?;
    assert_eq!(
        page.locator("[data-testid='catalog-price-unit']")
            .await
            .input_value(None)
            .await?,
        "per_square_foot",
        "the price unit persisted"
    );

    browser.close().await.context("close browser")?;
    Ok(())
}
