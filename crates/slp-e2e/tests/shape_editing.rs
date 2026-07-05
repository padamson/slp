//! F3.1 e2e: select a drawn area (mirroring how selecting a tree reveals its
//! canopy/trunk handles), move one of its nodes and watch the area change,
//! insert a node between two adjacent selected nodes, and delete a node.
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

/// Poll the shape label's `textContent` until it matches `expected` or a short
/// timeout elapses. `<text>` is an SVG element — `to_have_text`'s `innerText`
/// read (Playwright's usual assertion) throws on it, so this reads
/// `textContent` directly instead, with the same retry-until-timeout shape.
async fn expect_label(label: &Locator, expected: &str) -> Result<()> {
    let start = Instant::now();
    loop {
        let actual = label.text_content().await?.unwrap_or_default();
        if actual.trim() == expected {
            return Ok(());
        }
        if start.elapsed() >= Duration::from_secs(5) {
            return Err(anyhow!("expected shape label '{expected}', got '{actual}'"));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test]
async fn selects_a_shape_and_edits_its_nodes() -> Result<()> {
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
    expect_label(&label, "80 ft²")
        .await
        .context("the drawn rectangle is 10x8 ft")?;

    // Click inside its body (off any corner/label) to select it — no armed
    // tool, mirroring how clicking a tree selects it.
    click_ft(&yard, ppf, 12.0, 11.0).await?;
    expect(page.locator("[data-testid='yard'] .shape--selected").await)
        .to_have_count(1)
        .await
        .context("the shape is selected")?;
    expect(page.locator("[data-testid='shape-node']").await)
        .to_have_count(4)
        .await
        .context("its corners become interactive node handles")?;

    // Drag node 0 (world (10,10)) up to (10,12) — the quad's area shrinks
    // from 80 to 70 ft².
    let BoundingBox { x, y, .. } = yard
        .bounding_box()
        .await
        .context("measure the yard")?
        .context("yard has a bounding box")?;
    let screen = |fx: f64, fy: f64| (x + fx * ppf, y + (YARD_D - fy) * ppf);
    let mouse = page.mouse();
    let node0 = page.locator("[data-testid='shape-node']").await.nth(0);
    node0.hover(None).await.context("hover node 0")?;
    mouse.down(None).await.context("press node 0")?;
    let (mx, my) = screen(10.0, 12.0);
    mouse.move_to(mx as i32, my as i32, None).await.context("drag it up")?;
    mouse.up(None).await.context("release")?;
    expect_label(&label, "70 ft²")
        .await
        .context("moving a node changes the reported area live")?;

    // Select node 0 and the adjacent node 1 — the insert-between popup appears.
    page.locator("[data-testid='shape-node']")
        .await
        .nth(0)
        .click(None)
        .await
        .context("select node 0")?;
    page.locator("[data-testid='shape-node']")
        .await
        .nth(1)
        .click(None)
        .await
        .context("select adjacent node 1")?;
    expect(page.locator("[data-testid='insert-node']").await)
        .to_be_visible()
        .await
        .context("the insert-between popup appears for an adjacent pair")?;

    // Insert — the boundary gains a fifth node.
    page.locator("[data-testid='insert-node']")
        .await
        .click(None)
        .await
        .context("insert a node between them")?;
    expect(page.locator("[data-testid='shape-node']").await)
        .to_have_count(5)
        .await
        .context("the inserted node joins the boundary")?;

    // Select the newly-inserted node (index 1, between the original 0 and 1)
    // and delete it via Backspace — back down to 4 nodes.
    page.locator("[data-testid='shape-node']")
        .await
        .nth(1)
        .click(None)
        .await
        .context("select the inserted node")?;
    page.keyboard()
        .press("Backspace", None)
        .await
        .context("delete the selected node")?;
    expect(page.locator("[data-testid='shape-node']").await)
        .to_have_count(4)
        .await
        .context("the node is removed, back to the original boundary")?;

    browser.close().await.context("close browser")?;
    Ok(())
}
