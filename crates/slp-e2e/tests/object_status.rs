//! E1.4 e2e: toggling a placed object's cost status in the inspector changes how
//! it's drawn on the canvas — existing/virtual objects pick up a status class
//! (and, per `Furnishings`, a dashed outline + reduced opacity) so "not a
//! purchase" reads at a glance without opening the inspector.
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use anyhow::{Context, Result};
use common::{click_ft, dist_dir, draw_central_deck, measure_ppf, place, serve};
use playwright_rs::protocol::Playwright;
use playwright_rs::expect;

#[tokio::test]
async fn toggling_status_changes_the_footprints_class() -> Result<()> {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!("skipping: {} not built (run `trunk build`).", dist.display());
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
    draw_central_deck(&page, &yard, ppf).await?;
    let ppf = measure_ppf(&yard).await?;

    // Place a chair and select it — it starts planned (no status class).
    place(&page, &yard, ppf, 35.0, 15.0).await?;
    click_ft(&yard, ppf, 35.0, 15.0).await?; // select
    let footprint = page.locator("[data-testid='yard'] .furniture-item").await;
    expect(footprint.clone()).to_have_count(1).await.context("the object is on the plan")?;
    expect(
        page.locator("[data-testid='yard'] .furniture-item--existing")
            .await,
    )
    .to_have_count(0)
    .await
    .context("planned starts with no status class")?;

    // Existing: picks up the existing status class, and drops off virtual's.
    page.locator("[data-testid='status-existing']")
        .await
        .click(None)
        .await
        .context("click Existing")?;
    expect(
        page.locator("[data-testid='yard'] .furniture-item--existing")
            .await,
    )
    .to_have_count(1)
    .await
    .context("existing status shows on the canvas")?;

    // Virtual: swaps to the virtual status class.
    page.locator("[data-testid='status-virtual']")
        .await
        .click(None)
        .await
        .context("click Virtual")?;
    expect(
        page.locator("[data-testid='yard'] .furniture-item--existing")
            .await,
    )
    .to_have_count(0)
    .await
    .context("no longer carries the existing class")?;
    expect(
        page.locator("[data-testid='yard'] .furniture-item--virtual")
            .await,
    )
    .to_have_count(1)
    .await
    .context("virtual status shows on the canvas")?;

    // Back to planned: neither status class remains.
    page.locator("[data-testid='status-planned']")
        .await
        .click(None)
        .await
        .context("click Planned")?;
    expect(
        page.locator("[data-testid='yard'] .furniture-item--virtual")
            .await,
    )
    .to_have_count(0)
    .await
    .context("planned drops the virtual class")?;

    browser.close().await.context("close browser")?;
    Ok(())
}
