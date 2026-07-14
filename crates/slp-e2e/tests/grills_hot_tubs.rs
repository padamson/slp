//! D4/D5 e2e: place a grill (a rectangular appliance whose keep-clear zone
//! follows its rectangular shape, flagging when something's too close) and a
//! hot tub (a heavy unit that wants to sit on a surface).
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use anyhow::{Context, Result};
use common::{dist_dir, draw_central_deck, measure_ppf, place_object, serve};
use playwright_rs::expect;
use playwright_rs::protocol::Playwright;

#[tokio::test]
async fn a_grill_shows_a_rectangular_keep_clear_zone_that_flags_when_crowded() -> Result<()> {
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

    // A grill is in the catalog from the start (its own "Grill" palette group).
    expect(page.locator("[data-testid='palette-gas-grill']").await)
        .to_have_count(1)
        .await
        .context("the grill palette tile is available")?;

    // Place it — a rectangular footprint with a rounded-rect clearance zone.
    place_object(&page, &yard, ppf, "gas-grill", 20.0, 15.0).await?;
    let zone = page.locator("[data-testid='clearance-ring']").await;
    expect(zone.clone())
        .to_have_count(1)
        .await
        .context("the grill draws a clearance zone")?;
    assert!(
        zone.get_attribute("rx").await?.is_some(),
        "the zone is a rounded rectangle (has rx), not a circle"
    );

    // Isolated, the zone is quiet.
    expect(page.locator("[data-testid='yard'] .furniture-item--intrudes").await)
        .to_have_count(0)
        .await
        .context("nothing intrudes yet")?;

    // Drop a lounge chair just off the grill's east edge, inside the keep-clear.
    place_object(&page, &yard, ppf, "lounge-chair", 24.0, 15.0).await?;
    expect(page.locator("[data-testid='yard'] .furniture-item--intrudes").await)
        .to_have_count(1)
        .await
        .context("the grill zone flags the too-close chair")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn a_hot_tub_is_water_blue_and_flags_when_off_a_surface() -> Result<()> {
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
    draw_central_deck(&page, &yard, ppf).await?; // deck spans x:[28,42], y:[12,18]
    let ppf = measure_ppf(&yard).await?;

    // Place a round hot tub off the deck — it fills water blue and flags,
    // because a heavy tub belongs on a deck/paver.
    place_object(&page, &yard, ppf, "hot-tub-round", 10.0, 24.0).await?;
    let tub = page.locator("[data-testid='yard'] .furniture-item circle").await;
    expect(tub.clone())
        .to_have_count(1)
        .await
        .context("the round hot tub is drawn")?;
    assert_eq!(
        tub.get_attribute("fill").await?.as_deref(),
        Some("#6faec5"),
        "the hot tub fills water blue"
    );
    expect(page.locator("[data-testid='yard'] .furniture-item--overflows").await)
        .to_have_count(1)
        .await
        .context("a hot tub off a surface flags")?;

    browser.close().await.context("close browser")?;
    Ok(())
}
