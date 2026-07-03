//! F2.0 e2e: place objects from the palette. Clicking a tile arms that item;
//! clicking the canvas drops it. Covers the fire pit (round footprint), folding
//! in D2.0's deferred placement e2e now that it drives the palette flow.
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use anyhow::{Context, Result};
use common::{arm_object, click_ft, dist_dir, draw_central_deck, measure_ppf, place_object, serve};
use playwright_rs::protocol::Playwright;
use playwright_rs::expect;

#[tokio::test]
async fn arming_a_tile_then_clicking_places_the_object() -> Result<()> {
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

    // No palette until the catalog is seeded (by drawing a deck).
    expect(page.locator("[data-testid='object-palette']").await)
        .to_have_count(0)
        .await
        .context("no palette before a catalog exists")?;
    draw_central_deck(&page, &yard, ppf).await?;
    let ppf = measure_ppf(&yard).await?;
    expect(page.locator("[data-testid='object-palette']").await)
        .to_be_visible()
        .await
        .context("the palette appears once the catalog is seeded")?;

    // Arming a tile highlights it (and no object is placed yet).
    arm_object(&page, "lounge-chair").await?;
    expect(page.locator("[data-testid='palette-lounge-chair'].armed").await)
        .to_have_count(1)
        .await
        .context("the armed tile is highlighted")?;
    expect(page.locator("[data-testid='yard'] .furniture-item").await)
        .to_have_count(0)
        .await
        .context("arming alone places nothing")?;

    // Clicking the armed tile again disarms it.
    arm_object(&page, "lounge-chair").await?;
    expect(page.locator("[data-testid='palette-lounge-chair'].armed").await)
        .to_have_count(0)
        .await
        .context("clicking the armed tile cancels")?;

    // Place a chair (rect) and a fire pit (round) from the palette.
    place_object(&page, &yard, ppf, "lounge-chair", 32.0, 15.0).await?;
    place_object(&page, &yard, ppf, "fire-pit", 38.0, 15.0).await?;
    expect(page.locator("[data-testid='yard'] .furniture-item").await)
        .to_have_count(2)
        .await
        .context("both objects are on the plan")?;
    // The fire pit renders as a circle (rects are the chair + double-line etc.);
    // at least one <circle> footprint exists.
    expect(page.locator("[data-testid='yard'] .furniture-item circle").await)
        .to_have_count(1)
        .await
        .context("the fire pit renders a round footprint")?;
    // The estimate reflects both.
    expect(page.locator("[data-testid='estimate-total']").await)
        .to_be_visible()
        .await
        .context("the estimate has a total")?;

    // Selecting the fire pit shows its diameter in the inspector.
    click_ft(&yard, ppf, 38.0, 15.0).await?;
    expect(page.locator("[data-testid='object-inspector']:has-text('⌀ 3 ft')").await)
        .to_have_count(1)
        .await
        .context("the inspector shows the fire pit's diameter")?;

    browser.close().await.context("close browser")?;
    Ok(())
}
