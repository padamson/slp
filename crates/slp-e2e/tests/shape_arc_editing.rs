//! F3.2 e2e: draw a straight-edged area, select it, and drag one edge's bulge
//! handle to bow that edge into an arc — the boundary re-renders as a `<path>`
//! with an arc command and the reported area changes.
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

/// Poll a label's `textContent` until it stops equalling `stale` (i.e. it has
/// changed) or a short timeout elapses. `<text>` is an SVG element, so the
/// usual `innerText`-based assertions throw on it — read `textContent`.
async fn wait_until_label_changes(label: &Locator, stale: &str) -> Result<String> {
    let start = Instant::now();
    loop {
        let actual = label.text_content().await?.unwrap_or_default();
        let actual = actual.trim().to_string();
        if actual != stale {
            return Ok(actual);
        }
        if start.elapsed() >= Duration::from_secs(5) {
            return Err(anyhow!("label never changed from '{stale}'"));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test]
async fn bows_an_edge_into_an_arc() -> Result<()> {
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

    // Draw a 10x8 ft rectangle (80 ft²): edge 0 is the bottom (10,10)->(20,10).
    page.locator("[data-testid='draw-shape']")
        .click(None)
        .await
        .context("arm the shape tool")?;
    let corners = [(10.0, 10.0), (20.0, 10.0), (20.0, 18.0), (10.0, 18.0)];
    for (fx, fy) in corners {
        click_ft(&yard, ppf, fx, fy).await?;
    }
    click_ft(&yard, ppf, corners[0].0, corners[0].1).await?; // snap-close

    let label = page.locator("[data-testid='yard'] .shape-label");
    // Straight boundary renders as a polygon, area 80 ft².
    expect(page.locator("[data-testid='yard'] .shape polygon"))
        .to_have_count(1)
        .await
        .context("a straight boundary is a polygon")?;

    // Select the shape (click its body, off any corner/edge handle).
    click_ft(&yard, ppf, 12.0, 12.0).await?;
    expect(page.locator("[data-testid='shape-edge-handle']"))
        .to_have_count(4)
        .await
        .context("a bulge handle per edge appears when selected")?;

    // The bottom edge's handle starts at its midpoint (15,10). Drag it down to
    // (15,7) — bowing the edge outward, away from the interior → the area
    // grows past 80, and the boundary becomes an arc `<path>`.
    // The bottom-edge handle is the lowest-on-screen edge handle (largest y).
    let handles = page.locator("[data-testid='shape-edge-handle']");
    // Edge order is 0=bottom,1=right,2=top,3=left, so nth(0) is the bottom edge.
    let bottom = handles.nth(0);
    common::drag_to_ft(&bottom, &yard, ppf, 15.0, 7.0).await?;

    // The boundary re-rendered as a path with an arc command, and the area grew.
    expect(page.locator("[data-testid='yard'] .shape path"))
        .to_have_count(1)
        .await
        .context("the bowed boundary renders as a path")?;
    expect(page.locator("[data-testid='yard'] .shape polygon"))
        .to_have_count(0)
        .await
        .context("no polygon remains once the edge is an arc")?;
    let grown = wait_until_label_changes(&label, "80 ft²")
        .await
        .context("bowing the edge changed the reported area")?;
    let ft2: f64 = grown
        .trim_end_matches(" ft²")
        .parse()
        .with_context(|| format!("parse area from '{grown}'"))?;
    assert!(ft2 > 85.0, "the area grew past the straight 80 (got {ft2})");

    browser.close().await.context("close browser")?;
    Ok(())
}
