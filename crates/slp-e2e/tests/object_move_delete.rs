//! F1 e2e: direct manipulation of a placed object — pick-and-drag to move it, and
//! delete it (via the inspector's Remove button or the Delete key). We draw a
//! central deck (which seeds the furniture catalog), place a chair, then
//! manipulate it and assert the plan reflects the change.
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use anyhow::{Context, Result};
use common::{YARD_D, click_ft, dist_dir, draw_central_deck, measure_ppf, place, serve};
use playwright_rs::expect;
use playwright_rs::protocol::Playwright;

/// The app draws the plan in a viewBox of 12 px/ft with the origin flush to the
/// canvas and north up, so a footprint centered at world `(fx, fy)` renders with
/// `transform="translate(fx*12, (30-fy)*12) …"`. We assert against that translate
/// to read an object's committed world position straight off the SVG.
const VIEWBOX_PX_FT: f64 = 12.0;

fn translate_of(fx: f64, fy: f64) -> String {
    format!(
        "translate({},{})",
        fx * VIEWBOX_PX_FT,
        (YARD_D - fy) * VIEWBOX_PX_FT
    )
}

#[tokio::test]
async fn dragging_an_object_moves_it() -> Result<()> {
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
    draw_central_deck(&page, &yard, ppf).await?;

    // The estimate panel shrank the canvas — re-measure.
    let ppf = measure_ppf(&yard).await?;

    // Place a chair in the middle; it renders at (35, 15).
    let (from_x, from_y) = (35.0, 15.0);
    place(&page, &yard, ppf, from_x, from_y).await?;
    let start = translate_of(from_x, from_y);
    expect(page.locator(format!(
        "[data-testid='yard'] .furniture-item[transform*='{start}']"
    )))
    .to_have_count(1)
    .await
    .context("the object starts at its placed position")?;

    // Grab the object at its center and drag it to (45, 20). Snap-to-grid (on by
    // default) lands the center exactly on the foot grid at the drop point.
    let object = page.locator("[data-testid='yard'] .furniture-item");
    common::drag_to_ft(&object, &yard, ppf, 45.0, 20.0).await?;

    let dropped = translate_of(45.0, 20.0);
    expect(page.locator(format!(
        "[data-testid='yard'] .furniture-item[transform*='{dropped}']"
    )))
    .to_have_count(1)
    .await
    .context("dragging the object moves it to the drop point")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn remove_button_deletes_the_selected_object() -> Result<()> {
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
    draw_central_deck(&page, &yard, ppf).await?;
    let ppf = measure_ppf(&yard).await?;

    // Place a chair and select it — the estimate now has a line item.
    place(&page, &yard, ppf, 35.0, 15.0).await?;
    click_ft(&yard, ppf, 35.0, 15.0).await?; // select
    expect(page.locator("[data-testid='yard'] .furniture-item"))
        .to_have_count(1)
        .await
        .context("the object is on the plan")?;
    expect(page.locator("[data-testid='estimate-total']"))
        .to_be_visible()
        .await
        .context("the estimate has a total")?;

    // Remove it from the inspector.
    page.locator("[data-testid='delete-object']")
        .click(None)
        .await
        .context("click Remove")?;

    expect(page.locator("[data-testid='yard'] .furniture-item"))
        .to_have_count(0)
        .await
        .context("the footprint is gone")?;
    expect(page.locator("[data-testid='object-inspector']"))
        .to_have_count(0)
        .await
        .context("the inspector closes with the selection")?;
    expect(page.locator("[data-testid='estimate'] .estimate-empty"))
        .to_be_visible()
        .await
        .context("the estimate line drops")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn delete_key_removes_the_selected_object() -> Result<()> {
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
    draw_central_deck(&page, &yard, ppf).await?;
    let ppf = measure_ppf(&yard).await?;

    place(&page, &yard, ppf, 35.0, 15.0).await?;
    click_ft(&yard, ppf, 35.0, 15.0).await?; // select
    expect(page.locator("[data-testid='yard'] .furniture-item"))
        .to_have_count(1)
        .await
        .context("the object is selected")?;

    page.keyboard()
        .press("Delete", None)
        .await
        .context("press the Delete key")?;

    expect(page.locator("[data-testid='yard'] .furniture-item"))
        .to_have_count(0)
        .await
        .context("Delete removes the selected object")?;

    browser.close().await.context("close browser")?;
    Ok(())
}
