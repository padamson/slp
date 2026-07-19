//! B5 e2e: border rings around a drawn area. Selecting a paver patio offers a
//! border editor; adding a ring renders it inside the boundary and reprices
//! the estimate — a per-ft² ring itemizes as area, an edging stone (per
//! linear ft) as centerline feet, and the field's ft² shrinks by the band.
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

/// Poll until `loc`'s text does NOT contain `needle`, or time out.
async fn wait_absent(loc: &Locator, needle: &str) -> Result<()> {
    let start = Instant::now();
    loop {
        let text = loc.text_content().await?.unwrap_or_default();
        if !text.contains(needle) {
            return Ok(());
        }
        if start.elapsed() >= Duration::from_secs(5) {
            return Err(anyhow!("'{needle}' never went away; last was '{text}'"));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test]
async fn edges_a_paver_area_with_a_border_ring() -> Result<()> {
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

    // Arm Pavers, draw a 10×8 ft patio (80 ft², perimeter 36 ft), select it.
    page.locator("[data-testid='area-mat-cat-paver']")
        .click(None)
        .await
        .context("arm the Pavers material")?;
    page.locator("[data-testid='draw-shape']")
        .click(None)
        .await
        .context("arm the area tool")?;
    let corners = [(10.0, 10.0), (20.0, 10.0), (20.0, 18.0), (10.0, 18.0)];
    for (fx, fy) in corners {
        click_ft(&yard, ppf, fx, fy).await?;
    }
    click_ft(&yard, ppf, corners[0].0, corners[0].1).await?; // snap-close
    click_ft(&yard, ppf, 12.0, 11.0).await?; // select the patio

    // The inspector offers a border editor; the plain patio shows 80 ft² of
    // pavers and no rings.
    let estimate = page.locator("[data-testid='estimate']");
    wait_contains(&estimate, "80 ft²")
        .await
        .context("the plain field's paver line")?;
    expect(page.locator("[data-testid='border-editor']"))
        .to_be_visible()
        .await
        .context("selecting the area offers the border editor")?;
    expect(page.locator("[data-testid='shape-border']"))
        .to_have_count(0)
        .await
        .context("no rings drawn yet")?;

    // Add a ring, then make it the edging stone (per linear ft). Width stays
    // the added default 0.5 ft: centerline = 36 − 2π·0.25 ≈ 34.4 lf, band
    // ≈ 17.2 ft² — so the paver field drops to ≈ 63 ft².
    page.locator("[data-testid='border-add']")
        .click(None)
        .await
        .context("add a border ring")?;
    expect(page.locator("[data-testid='border-row-0']"))
        .to_have_count(1)
        .await
        .context("the ring shows in the editor")?;
    expect(page.locator("[data-testid='shape-border']"))
        .to_have_count(1)
        .await
        .context("the ring renders inside the boundary")?;
    page.locator("[data-testid='border-material']")
        .select_option("edging-stone", None)
        .await
        .context("edge with the edging stone")?;
    wait_contains(&estimate, "Edging stones")
        .await
        .context("the edging line appears")?;
    wait_contains(&estimate, "34 lf")
        .await
        .context("costed by centerline linear feet")?;
    wait_contains(&estimate, "63 ft²")
        .await
        .context("the paver field shrinks by the band")?;

    // Scope the border to a node span: From n0 / To n2 covers edges 0 and 1
    // (the 10 ft south + 8 ft east sides = 18 lf, an open run with no corner
    // shrink); the field only loses that band (80 − 9 = 71 ft²), and the ring
    // renders as an open sub-path.
    page.locator("[data-testid='border-from']")
        .select_option("0", None)
        .await
        .context("span from node 0")?;
    page.locator("[data-testid='border-to']")
        .select_option("2", None)
        .await
        .context("span to node 2")?;
    wait_contains(&estimate, "18 lf")
        .await
        .context("the span's edges only")?;
    wait_contains(&estimate, "71 ft²")
        .await
        .context("the field loses only the span band")?;
    expect(page.locator("[data-testid='shape-border']"))
        .to_have_count(1)
        .await
        .context("the span band renders")?;

    // Remove the ring — the field is whole again and the edging line leaves.
    page.locator("[data-testid='border-remove-0']")
        .click(None)
        .await
        .context("remove the ring")?;
    wait_absent(&estimate, "Edging stones")
        .await
        .context("the edging line drops with its ring")?;
    wait_contains(&estimate, "80 ft²")
        .await
        .context("the field is back to its full area")?;
    expect(page.locator("[data-testid='shape-border']"))
        .to_have_count(0)
        .await
        .context("no ring rendered after removal")?;

    browser.close().await.context("close browser")?;
    Ok(())
}
