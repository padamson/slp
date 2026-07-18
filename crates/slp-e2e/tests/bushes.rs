//! D3 e2e: place a bush from the palette (available immediately — a shrub goes
//! in the yard, no deck needed), see its green round canopy + a cost line, and
//! watch its whole footprint flag red when it's dropped on hardscape (deck/
//! house) instead of open ground.
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use common::{dist_dir, draw_central_deck, measure_ppf, place_object, serve};
use playwright_rs::protocol::Playwright;
use playwright_rs::{Locator, expect};

/// Poll a locator's `text_content` until it contains `needle` or times out.
async fn wait_contains(loc: &Locator, needle: &str) -> Result<String> {
    let start = Instant::now();
    loop {
        let text = loc.text_content().await?.unwrap_or_default();
        if text.contains(needle) {
            return Ok(text);
        }
        if start.elapsed() >= Duration::from_secs(5) {
            return Err(anyhow!("'{needle}' never appeared; last was '{text}'"));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test]
async fn placing_a_bush_shows_a_green_canopy_and_costs_it() -> Result<()> {
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

    // A bush is in the catalog from the start (its own "Bush" palette group),
    // no deck required.
    expect(page.locator("[data-testid='palette-boxwood']"))
        .to_have_count(1)
        .await
        .context("the bush palette tile is available immediately")?;

    // Place it on bare yard; it renders a round green canopy.
    place_object(&page, &yard, ppf, "boxwood", 12.0, 15.0).await?;
    let canopy = page.locator("[data-testid='yard'] .furniture-item circle");
    expect(canopy.clone())
        .to_have_count(1)
        .await
        .context("the bush is a single round canopy (no trunk)")?;
    assert_eq!(
        canopy.get_attribute("fill").await?.as_deref(),
        Some("#7cae83"),
        "the bush fills its green"
    );

    // It's costed like any placed object — a Boxwood line in the estimate.
    let estimate = page.locator("[data-testid='estimate']");
    wait_contains(&estimate, "Boxwood")
        .await
        .context("the estimate lists the bush")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn a_bush_flags_red_only_on_hardscape() -> Result<()> {
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
    draw_central_deck(&page, &yard, ppf).await?; // deck spans x:[28,42], y:[12,18]
    let ppf = measure_ppf(&yard).await?;

    // Place the bush on bare yard, well off the deck.
    place_object(&page, &yard, ppf, "boxwood", 10.0, 24.0).await?;
    let bush = page.locator("[data-testid='yard'] .furniture-item");
    expect(page.locator("[data-testid='yard'] .furniture-item--overflows"))
        .to_have_count(0)
        .await
        .context("a bush on bare yard is fine")?;

    // Drag it onto the deck — its whole footprint should flag red.
    common::drag_to_ft(&bush, &yard, ppf, 35.0, 15.0).await?;
    expect(page.locator("[data-testid='yard'] .furniture-item--overflows"))
        .to_have_count(1)
        .await
        .context("the bush flags on the deck")?;

    // Drag it back off the deck — it should clear.
    common::drag_to_ft(&bush, &yard, ppf, 10.0, 24.0).await?;
    expect(page.locator("[data-testid='yard'] .furniture-item--overflows"))
        .to_have_count(0)
        .await
        .context("the bush clears back on bare ground")?;

    browser.close().await.context("close browser")?;
    Ok(())
}
