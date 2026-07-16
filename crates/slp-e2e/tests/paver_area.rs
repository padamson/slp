//! B1 e2e: arm the Pavers material, draw a paver area, and confirm it renders
//! in the paver look and shows a per-ft² Pavers line in the estimate — and
//! that it persists across a reload.
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use common::{click_ft, dist_dir, measure_ppf, serve};
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
async fn draws_a_paver_area_and_costs_it_per_square_foot() -> Result<()> {
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

    let yard = page.locator("[data-testid='yard']").await;
    let ppf = measure_ppf(&yard).await?;

    // Arm the Pavers material, then draw a 10×8 ft area (80 ft²).
    page.locator("[data-testid='area-mat-cat-paver']")
        .await
        .click(None)
        .await
        .context("arm the Pavers material")?;
    page.locator("[data-testid='draw-shape']")
        .await
        .click(None)
        .await
        .context("arm the area tool")?;
    let corners = [(10.0, 10.0), (20.0, 10.0), (20.0, 18.0), (10.0, 18.0)];
    for (fx, fy) in corners {
        click_ft(&yard, ppf, fx, fy).await?;
    }
    click_ft(&yard, ppf, corners[0].0, corners[0].1).await?; // snap-close

    // The area renders in the paver look (a gray fill, not mulch brown).
    let poly = page.locator("[data-testid='yard'] .shape polygon").await;
    expect(poly.clone())
        .to_have_count(1)
        .await
        .context("the paver area is drawn")?;
    let fill = poly
        .get_attribute("fill")
        .await
        .context("read the paver fill")?
        .context("polygon has a fill")?;
    assert_eq!(fill, "#9a9ca0", "the area fills paver gray");

    // The estimate shows a Pavers line reading ft² (80 × $8 = $640).
    let estimate = page.locator("[data-testid='estimate']").await;
    let text = wait_contains(&estimate, "Pavers")
        .await
        .context("the estimate lists a Pavers line")?;
    assert!(
        text.contains("ft²"),
        "the paver quantity reads in ft²: {text}"
    );
    assert!(text.contains("80 ft²"), "10×8 = 80 ft²: {text}");

    // The paver assembly itemizes its sub-base: a Gravel base and a Bedding
    // sand line, each measured in yd³ (80 ft² × 4 in / 324 ≈ 1.0 yd³ gravel,
    // × 1 in ≈ 0.2 yd³ sand).
    let with_base = wait_contains(&estimate, "Gravel base")
        .await
        .context("the estimate itemizes the gravel base")?;
    assert!(
        with_base.contains("Bedding sand"),
        "and the bedding sand: {with_base}"
    );
    assert!(
        with_base.contains("yd³"),
        "the sub-base courses read in yd³: {with_base}"
    );

    // Reload — the paver area (and its cost) persist.
    page.reload(None).await.context("reload the page")?;
    expect(page.locator("[data-testid='yard'] .shape polygon").await)
        .to_have_count(1)
        .await
        .context("the paver area persists across a reload")?;
    wait_contains(&page.locator("[data-testid='estimate']").await, "Pavers")
        .await
        .context("the Pavers line persists across a reload")?;

    browser.close().await.context("close browser")?;
    Ok(())
}
