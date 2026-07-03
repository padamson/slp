//! E1.6 e2e: the inspector's two independent controls — an existing/planned
//! status toggle and a real/virtual toggle — change how the object is drawn on
//! the canvas. Existing gets a double outline (vs. planned's single); virtual
//! gets a dashed, ghosted look (vs. real's solid, full color) — so both status
//! and realness read at a glance without reopening the inspector.
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
async fn status_and_virtual_toggles_change_the_footprints_class() -> Result<()> {
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

    // Place a chair and select it — it starts planned + real.
    place(&page, &yard, ppf, 35.0, 15.0).await?;
    click_ft(&yard, ppf, 35.0, 15.0).await?; // select
    expect(page.locator("[data-testid='yard'] .furniture-item").await)
        .to_have_count(1)
        .await
        .context("the object is on the plan")?;
    expect(page.locator("[data-testid='yard'] .furniture-item--planned").await)
        .to_have_count(1)
        .await
        .context("starts planned")?;
    expect(page.locator("[data-testid='yard'] .furniture-item--existing").await)
        .to_have_count(0)
        .await
        .context("not existing")?;
    expect(page.locator("[data-testid='yard'] .furniture-item--virtual").await)
        .to_have_count(0)
        .await
        .context("not virtual")?;

    // Existing: swaps the status class; still real (no virtual class).
    page.locator("[data-testid='status-existing']")
        .await
        .click(None)
        .await
        .context("click Existing")?;
    expect(page.locator("[data-testid='yard'] .furniture-item--existing").await)
        .to_have_count(1)
        .await
        .context("existing status shows on the canvas")?;
    expect(page.locator("[data-testid='yard'] .furniture-item--planned").await)
        .to_have_count(0)
        .await
        .context("no longer planned")?;
    expect(page.locator("[data-testid='yard'] .furniture-item--virtual").await)
        .to_have_count(0)
        .await
        .context("still real — the virtual toggle is untouched")?;

    // Virtual toggle: an independent control — existing stays, virtual joins.
    page.locator("[data-testid='inspector-virtual']")
        .await
        .click(None)
        .await
        .context("toggle Virtual on")?;
    expect(page.locator("[data-testid='yard'] .furniture-item--existing").await)
        .to_have_count(1)
        .await
        .context("still existing")?;
    expect(page.locator("[data-testid='yard'] .furniture-item--virtual").await)
        .to_have_count(1)
        .await
        .context("virtual joins independently of status")?;

    // Status flips back to planned while virtual stays on — the two controls
    // are independent, not a single 3-way choice.
    page.locator("[data-testid='status-planned']")
        .await
        .click(None)
        .await
        .context("click Planned")?;
    expect(page.locator("[data-testid='yard'] .furniture-item--planned").await)
        .to_have_count(1)
        .await
        .context("back to planned")?;
    expect(page.locator("[data-testid='yard'] .furniture-item--existing").await)
        .to_have_count(0)
        .await
        .context("no longer existing")?;
    expect(page.locator("[data-testid='yard'] .furniture-item--virtual").await)
        .to_have_count(1)
        .await
        .context("virtual is untouched by the status change")?;

    // Toggle virtual back off: real, planned.
    page.locator("[data-testid='inspector-virtual']")
        .await
        .click(None)
        .await
        .context("toggle Virtual off")?;
    expect(page.locator("[data-testid='yard'] .furniture-item--virtual").await)
        .to_have_count(0)
        .await
        .context("virtual toggles off independently")?;

    browser.close().await.context("close browser")?;
    Ok(())
}
