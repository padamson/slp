//! F3.3 e2e: draw a straight-edged area, select it, and drag a Bézier control
//! handle to curve one edge — the boundary re-renders as a `<path>` with a `C`
//! command and the reported area changes.
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use common::{YARD_D, click_ft, dist_dir, measure_ppf, serve};
use playwright_rs::protocol::Playwright;
use playwright_rs::{BoundingBox, Locator, expect};

/// Poll a label's `textContent` until it stops equalling `stale`, returning the
/// new text. `<text>` is an SVG element, so `innerText` assertions throw — read
/// `textContent`.
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
async fn curves_an_edge_with_a_bezier_control_handle() -> Result<()> {
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
    let page = browser.new_page().await.context("new page")?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;

    let yard = page.locator("[data-testid='yard']").await;
    let ppf = measure_ppf(&yard).await?;

    // Draw a 10x8 ft rectangle (80 ft²).
    page.locator("[data-testid='draw-shape']")
        .await
        .click(None)
        .await
        .context("arm the shape tool")?;
    let corners = [(10.0, 10.0), (20.0, 10.0), (20.0, 18.0), (10.0, 18.0)];
    for (fx, fy) in corners {
        click_ft(&yard, ppf, fx, fy).await?;
    }
    click_ft(&yard, ppf, corners[0].0, corners[0].1).await?; // snap-close

    let label = page.locator("[data-testid='yard'] .shape-label").await;
    expect(page.locator("[data-testid='yard'] .shape polygon").await)
        .to_have_count(1)
        .await
        .context("a straight boundary is a polygon")?;

    // Select the shape → control handles appear (2 per edge = 8).
    click_ft(&yard, ppf, 12.0, 12.0).await?;
    expect(page.locator("[data-testid='shape-control-handle']").await)
        .to_have_count(8)
        .await
        .context("two control handles per straight edge")?;

    // The bottom edge (edge 0, node 0->1) has its control handles at the chord
    // thirds: (13.33,10) and (16.67,10). Grab the first (control1 of edge 0)
    // and drag it down to (13,6) — promoting edge 0 to a bezier that bows out,
    // growing the area and re-rendering the boundary as a path with a `C`.
    let BoundingBox { x, y, .. } = yard
        .bounding_box()
        .await
        .context("measure the yard")?
        .context("yard has a bounding box")?;
    let screen = |fx: f64, fy: f64| (x + fx * ppf, y + (YARD_D - fy) * ppf);
    let mouse = page.mouse();
    // Control handles render in edge order; nth(0) is edge 0's control1.
    let control0 = page
        .locator("[data-testid='shape-control-handle']")
        .await
        .nth(0);
    control0
        .hover(None)
        .await
        .context("hover control1 of edge 0")?;
    mouse.down(None).await.context("press the control handle")?;
    let (mx, my) = screen(13.0, 6.0);
    mouse
        .move_to(mx as i32, my as i32, None)
        .await
        .context("drag it down")?;
    mouse.up(None).await.context("release")?;

    expect(page.locator("[data-testid='yard'] .shape path").await)
        .to_have_count(1)
        .await
        .context("the curved boundary renders as a path")?;
    expect(page.locator("[data-testid='yard'] .shape polygon").await)
        .to_have_count(0)
        .await
        .context("no polygon remains once an edge is a curve")?;
    let grown = wait_until_label_changes(&label, "80 ft²")
        .await
        .context("curving the edge changed the reported area")?;
    let ft2: f64 = grown
        .trim_end_matches(" ft²")
        .parse()
        .with_context(|| format!("parse area from '{grown}'"))?;
    assert!(ft2 > 85.0, "the area grew past the straight 80 (got {ft2})");

    // Persists across a reload.
    page.reload(None).await.context("reload the page")?;
    expect(page.locator("[data-testid='yard'] .shape path").await)
        .to_have_count(1)
        .await
        .context("the curved boundary persists across a reload")?;

    browser.close().await.context("close browser")?;
    Ok(())
}
