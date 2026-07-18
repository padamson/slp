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
use common::{arm_object, click_ft, dist_dir, measure_ppf, place_object, serve};
use playwright_rs::expect;
use playwright_rs::protocol::Playwright;

#[tokio::test]
async fn arming_a_tile_then_clicking_places_the_object() -> Result<()> {
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
    let page = common::new_page(&browser).await?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;

    let yard = page.locator("[data-testid='yard']");
    let ppf = measure_ppf(&yard).await?;

    // The palette is available immediately — the starter catalog is seeded on
    // load, no deck required (a fire pit or a tree doesn't need one).
    expect(page.locator("[data-testid='object-palette']"))
        .to_be_visible()
        .await
        .context("the palette is available without drawing anything first")?;

    // Arming a tile highlights it (and no object is placed yet).
    arm_object(&page, "lounge-chair").await?;
    expect(page.locator("[data-testid='palette-lounge-chair'].armed"))
        .to_have_count(1)
        .await
        .context("the armed tile is highlighted")?;
    expect(page.locator("[data-testid='yard'] .furniture-item"))
        .to_have_count(0)
        .await
        .context("arming alone places nothing")?;

    // Clicking the armed tile again disarms it.
    arm_object(&page, "lounge-chair").await?;
    expect(page.locator("[data-testid='palette-lounge-chair'].armed"))
        .to_have_count(0)
        .await
        .context("clicking the armed tile cancels")?;

    // Place a chair (rect) and a fire pit (round) from the palette.
    place_object(&page, &yard, ppf, "lounge-chair", 32.0, 15.0).await?;
    place_object(&page, &yard, ppf, "fire-pit", 38.0, 15.0).await?;
    expect(page.locator("[data-testid='yard'] .furniture-item"))
        .to_have_count(2)
        .await
        .context("both objects are on the plan")?;
    // The fire pit renders as a circle (rects are the chair + double-line etc.);
    // exclude the clearance ring (also a `<circle>`) so this counts only
    // footprints.
    expect(page.locator(
        "[data-testid='yard'] .furniture-item circle:not([data-testid='clearance-ring'])",
    ))
    .to_have_count(1)
    .await
    .context("the fire pit renders a round footprint")?;
    // The estimate reflects both.
    expect(page.locator("[data-testid='estimate-total']"))
        .to_be_visible()
        .await
        .context("the estimate has a total")?;

    // Selecting the fire pit shows its diameter in the inspector.
    click_ft(&yard, ppf, 38.0, 15.0).await?;
    expect(page.locator("[data-testid='object-inspector']:has-text('⌀ 3 ft')"))
        .to_have_count(1)
        .await
        .context("the inspector shows the fire pit's diameter")?;

    browser.close().await.context("close browser")?;
    Ok(())
}
