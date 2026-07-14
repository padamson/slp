//! H1 e2e: draw the house outline with the node-placement engine — hover shows
//! the next node, a click commits it, clicking the first corner closes the ring;
//! and a press-drag-release drops a node at the release point. Confirms the
//! committed outline persists across a reload.
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use anyhow::{Context, Result};
use common::{dist_dir, serve};
use playwright_rs::protocol::Playwright;
use playwright_rs::{ClickOptions, Locator, Position, expect};

async fn click_yard(yard: &Locator, x: f64, y: f64) -> Result<()> {
    let opts = ClickOptions::builder().position(Position { x, y }).build();
    yard.click(Some(opts)).await.context("click the yard")?;
    Ok(())
}

#[tokio::test]
async fn draws_and_persists_the_house_outline() -> Result<()> {
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

    // Arm the house tool.
    page.locator("[data-testid='draw-house']")
        .await
        .click(None)
        .await
        .context("arm the house tool")?;

    // Click four corners — each lands as an in-progress placement node.
    let yard = page.locator("[data-testid='yard']").await;
    let first = (120.0, 120.0);
    let corners = [first, (360.0, 120.0), (360.0, 300.0), (120.0, 300.0)];
    for (x, y) in corners {
        click_yard(&yard, x, y).await?;
    }
    expect(page.locator("[data-testid='yard'] .placement-node").await)
        .to_have_count(4)
        .await
        .context("four in-progress nodes placed")?;

    // Click the first corner to close — the placement becomes the committed house.
    click_yard(&yard, first.0, first.1).await?;
    expect(page.locator("[data-testid='yard'] .house-corner").await)
        .to_have_count(4)
        .await
        .context("the closed outline has four corners")?;

    // Snapping is on by default → the outline polygon's coordinates are whole
    // user-space px (12·feet; the grid is flush to the canvas, no padding).
    let points = page
        .locator("[data-testid='yard'] .house polygon")
        .await
        .get_attribute("points")
        .await
        .context("read the polygon points")?
        .context("polygon has a points attribute")?;
    for n in points.split([' ', ',']).filter(|s| !s.is_empty()) {
        let v: f64 = n.parse().context("parse a polygon coordinate")?;
        assert!(v.fract().abs() < 1e-6, "corner snapped to the grid: {v}");
    }

    // Reload — the committed house is restored from localStorage.
    page.reload(None).await.context("reload the page")?;
    expect(page.locator("[data-testid='yard'] .house-corner").await)
        .to_have_count(4)
        .await
        .context("the drawn house persists across a reload")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn holds_to_adjust_and_drops_on_release() -> Result<()> {
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

    page.locator("[data-testid='draw-house']")
        .await
        .click(None)
        .await
        .context("arm the house tool")?;

    let yard = page.locator("[data-testid='yard']").await;
    let bbox = yard
        .bounding_box()
        .await
        .context("measure the yard")?
        .context("yard has a bounding box")?;

    // Default 70 ft yard → viewBox width = 70·12 = 840 (grid flush to the canvas,
    // no padding). Map a page-x to the snapped SVG user-space x a dropped node
    // would get (grid step 1 ft).
    let scale = 840.0 / bbox.width;
    let snapped_cx = |page_x: f64| -> f64 {
        let ux = (page_x - bbox.x) * scale;
        let feet = (ux / 12.0).round();
        feet * 12.0
    };

    // Press at A, drag to B, release at B — the node drops at B.
    let (ax, ay) = (bbox.x + 120.0, bbox.y + 200.0);
    let (bx, by) = (bbox.x + 380.0, bbox.y + 200.0);
    let mouse = page.mouse();
    mouse.move_to(ax as i32, ay as i32, None).await.context("aim at A")?;
    mouse.down(None).await.context("press")?;
    mouse.move_to(bx as i32, by as i32, None).await.context("drag to B")?;
    mouse.up(None).await.context("release at B")?;

    let marker = page.locator("[data-testid='yard'] .placement-node").await;
    expect(marker.clone()).to_have_count(1).await.context("one node dropped")?;
    let cx: f64 = marker
        .get_attribute("cx")
        .await
        .context("read the node cx")?
        .context("node has a cx")?
        .parse()
        .context("parse cx")?;
    assert!(
        (cx - snapped_cx(bx)).abs() < 0.5,
        "node dropped at the release point B (cx={cx}, expected {})",
        snapped_cx(bx)
    );
    assert!(
        (cx - snapped_cx(ax)).abs() > 1.0,
        "node did NOT drop at the press point A (cx={cx}, A maps to {})",
        snapped_cx(ax)
    );

    browser.close().await.context("close browser")?;
    Ok(())
}
