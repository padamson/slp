//! E1.3 e2e: the object-inspector window floats in the first *empty* yard corner
//! (priority NE → NW → SE → SW, falling back to NE when all are occupied), plus
//! the drag-to-rotate handle. We draw a central deck (which seeds the furniture
//! catalog), place a target chair in the middle, then fill corners one at a time
//! and re-select the target, asserting the window hops to the next free corner
//! each time; and separately, drag the selected object's handle to rotate it.
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use anyhow::{Context, Result};
use common::{YARD_D, click_ft, dist_dir, draw_central_deck, measure_ppf, place, serve};
use playwright_rs::protocol::Playwright;
use playwright_rs::{BoundingBox, expect};

/// Assert the inspector is showing in `corner` (`nw`/`sw`/`ne`/`se`).
async fn assert_corner(page: &playwright_rs::Page, corner: &str) -> Result<()> {
    let sel = format!("[data-testid='object-inspector'][data-corner='{corner}']");
    expect(page.locator(&sel).await)
        .to_have_count(1)
        .await
        .with_context(|| format!("inspector floats in the {corner} corner"))?;
    Ok(())
}

#[tokio::test]
async fn inspector_floats_in_the_first_empty_corner() -> Result<()> {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!("skipping: {} not built (run `trunk build`).", dist.display());
        return Ok(());
    }

    let (addr, _server) = serve(&dist).await?;
    let pw = Playwright::launch().await.context("launch playwright")?;
    let browser = pw.chromium().launch().await.context("launch chromium")?;
    let page = common::new_page(&browser).await?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;

    let yard = page.locator("[data-testid='yard']").await;
    // Full-width canvas here (the estimate panel only appears once furniture is
    // seeded), so this ppf is right for the deck.
    let ppf = measure_ppf(&yard).await?;

    draw_central_deck(&page, &yard, ppf).await?;

    // Seeding the catalog made the estimate panel appear, shrinking the canvas —
    // re-measure so furniture clicks land in the (now narrower) yard.
    let ppf = measure_ppf(&yard).await?;

    // Place a target chair in the middle (on the deck) and select it. Priority is
    // NE → NW → SE → SW, falling back to NE when all four corners are occupied.
    let (tx, ty) = (35.0, 15.0);
    place(&page, &yard, ppf, tx, ty).await?;
    click_ft(&yard, ppf, tx, ty).await?; // no tool armed → selects
    assert_corner(&page, "ne").await?; // every corner empty → NE

    // Fill NE → NW.
    place(&page, &yard, ppf, 65.0, 25.0).await?;
    click_ft(&yard, ppf, tx, ty).await?;
    assert_corner(&page, "nw").await?;

    // Fill NW → SE.
    place(&page, &yard, ppf, 5.0, 25.0).await?;
    click_ft(&yard, ppf, tx, ty).await?;
    assert_corner(&page, "se").await?;

    // Fill SE → SW.
    place(&page, &yard, ppf, 65.0, 5.0).await?;
    click_ft(&yard, ppf, tx, ty).await?;
    assert_corner(&page, "sw").await?;

    // Fill SW → all four occupied, falls back to NE.
    place(&page, &yard, ppf, 5.0, 5.0).await?;
    click_ft(&yard, ppf, tx, ty).await?;
    assert_corner(&page, "ne").await?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn dragging_the_handle_rotates_the_object() -> Result<()> {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!("skipping: {} not built (run `trunk build`).", dist.display());
        return Ok(());
    }

    let (addr, _server) = serve(&dist).await?;
    let pw = Playwright::launch().await.context("launch playwright")?;
    let browser = pw.chromium().launch().await.context("launch chromium")?;
    let page = common::new_page(&browser).await?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;

    let yard = page.locator("[data-testid='yard']").await;
    let ppf = measure_ppf(&yard).await?;

    draw_central_deck(&page, &yard, ppf).await?;

    // Re-measure after the estimate panel appears, and grab the yard's screen box.
    let ppf = measure_ppf(&yard).await?;
    let BoundingBox { x, y, .. } = yard
        .bounding_box()
        .await
        .context("measure the yard")?
        .context("yard has a bounding box")?;

    // Place a chair in the middle and select it.
    let (cx_ft, cy_ft) = (35.0, 15.0);
    place(&page, &yard, ppf, cx_ft, cy_ft).await?;
    click_ft(&yard, ppf, cx_ft, cy_ft).await?; // select
    expect(page.locator("[data-testid='yard'] .furniture-item[transform*='rotate(0)']").await)
        .to_have_count(1)
        .await
        .context("the object starts un-rotated")?;

    // Grab the rotation handle and drag due east of the object's center — its
    // north edge turns to face the cursor, which snaps to 90°.
    page.locator("[data-testid='rotate-handle']")
        .await
        .hover(None)
        .await
        .context("hover the rotation handle")?;
    let mouse = page.mouse();
    mouse.down(None).await.context("press the handle")?;
    let center_x = x + cx_ft * ppf;
    let center_y = y + (YARD_D - cy_ft) * ppf;
    mouse
        .move_to((center_x + 120.0) as i32, center_y as i32, None)
        .await
        .context("drag east")?;
    mouse.up(None).await.context("release")?;

    expect(page.locator("[data-testid='yard'] .furniture-item[transform*='rotate(90)']").await)
        .to_have_count(1)
        .await
        .context("dragging the handle east rotates the object to 90°")?;

    browser.close().await.context("close browser")?;
    Ok(())
}
