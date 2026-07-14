//! B4 e2e: draw a mulch bed, set its depth, and confirm the estimate shows a
//! mulch line (yd³ × $/yd³) — and that it persists across a reload.
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
async fn draws_a_mulch_bed_and_costs_its_volume() -> Result<()> {
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

    // Set the mulch depth to 3 in (the default, but set it explicitly).
    let depth = page.locator("[data-testid='area-depth']").await;
    depth
        .fill("3", None)
        .await
        .context("set mulch depth to 3 in")?;

    // Draw a 10×8 ft mulch bed (80 ft²). At 3 in deep: yd³ = 80·3/324 ≈ 0.74;
    // × $40/yd³ ≈ $29.63.
    page.locator("[data-testid='draw-shape']")
        .await
        .click(None)
        .await
        .context("arm the bed tool")?;
    let corners = [(10.0, 10.0), (20.0, 10.0), (20.0, 18.0), (10.0, 18.0)];
    for (fx, fy) in corners {
        click_ft(&yard, ppf, fx, fy).await?;
    }
    click_ft(&yard, ppf, corners[0].0, corners[0].1).await?; // snap-close

    // The bed renders (a filled area on the plan).
    expect(page.locator("[data-testid='yard'] .shape polygon").await)
        .to_have_count(1)
        .await
        .context("the mulch bed is drawn")?;

    // The estimate gains a mulch line reading yd³, and a non-zero total.
    let estimate = page.locator("[data-testid='estimate']").await;
    let text = wait_contains(&estimate, "Mulch")
        .await
        .context("the estimate lists a Mulch line")?;
    assert!(
        text.contains("yd³"),
        "the mulch quantity reads in yd³: {text}"
    );
    let total = page.locator("[data-testid='estimate-total']").await;
    let total_text = total.text_content().await?.unwrap_or_default();
    assert!(
        total_text.starts_with('$') && total_text != "$0.00",
        "the grand total reflects the mulch cost (got {total_text})"
    );

    // Reload — the bed (and its cost) persist.
    page.reload(None).await.context("reload the page")?;
    expect(page.locator("[data-testid='yard'] .shape polygon").await)
        .to_have_count(1)
        .await
        .context("the mulch bed persists across a reload")?;
    wait_contains(&page.locator("[data-testid='estimate']").await, "Mulch")
        .await
        .context("the mulch line persists across a reload")?;

    browser.close().await.context("close browser")?;
    Ok(())
}
