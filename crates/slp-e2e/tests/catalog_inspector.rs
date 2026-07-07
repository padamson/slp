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
        if let Some(v) = loc.get_attribute(attr).await?.and_then(|s| s.parse().ok()) {
            if pred(v) {
                return Ok(v);
            }
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
