//! H1.1 e2e: draw the house outline by clicking corners, close it by clicking
//! near the first corner, and confirm it persists across a reload.
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

/// Click the yard stage at an offset (CSS px) within the SVG element.
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
    let page = browser.new_page().await.context("new page")?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;

    // Enter draw mode.
    page.locator("[data-testid='draw-house']")
        .await
        .click(None)
        .await
        .context("click Draw house")?;
    expect(page.locator("[data-testid='draw-house']").await)
        .to_contain_text("Click near the start to finish")
        .await
        .context("the button shows we're drawing")?;

    // Click four corners of a rough rectangle inside the stage.
    let yard = page.locator("[data-testid='yard']").await;
    let first = (120.0, 120.0);
    let corners = [first, (360.0, 120.0), (360.0, 300.0), (120.0, 300.0)];
    for (x, y) in corners {
        click_yard(&yard, x, y).await?;
    }
    expect(page.locator("[data-testid='yard'] .house-corner").await)
        .to_have_count(4)
        .await
        .context("a marker landed at each clicked corner")?;
    expect(page.locator("[data-testid='yard'] .house polygon").await)
        .to_have_count(1)
        .await
        .context("the outline polygon is drawn")?;

    // Snapping is on by default, so every corner lands on the grid → its SVG
    // user-space coordinates (40 + 12·feet) are whole numbers.
    let points = page
        .locator("[data-testid='yard'] .house polygon")
        .await
        .get_attribute("points")
        .await
        .context("read the polygon points")?
        .context("polygon has a points attribute")?;
    for n in points.split([' ', ',']).filter(|s| !s.is_empty()) {
        let v: f64 = n.parse().context("parse a polygon coordinate")?;
        assert!(
            (v.fract()).abs() < 1e-6,
            "corner snapped to the grid (whole user-space px): {v}"
        );
    }

    // Close by clicking near the first corner; drawing mode ends.
    click_yard(&yard, first.0, first.1).await?;
    expect(page.locator("[data-testid='draw-house']").await)
        .to_contain_text("Draw house")
        .await
        .context("closing the ring exits draw mode")?;
    expect(page.locator("[data-testid='yard'] .house-corner").await)
        .to_have_count(4)
        .await
        .context("the closed outline still has its four corners")?;

    // Reload — the house is restored from localStorage.
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
    let page = browser.new_page().await.context("new page")?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;

    page.locator("[data-testid='draw-house']")
        .await
        .click(None)
        .await
        .context("enter draw mode")?;

    let yard = page.locator("[data-testid='yard']").await;
    let bbox = yard
        .bounding_box()
        .await
        .context("measure the yard")?
        .context("yard has a bounding box")?;

    // Default 70 ft yard → viewBox width = 70·12 + 2·40 = 920. Map a page-x to
    // the snapped SVG user-space x a dropped corner would get (grid step 1 ft).
    let scale = 920.0 / bbox.width;
    let snapped_cx = |page_x: f64| -> f64 {
        let ux = (page_x - bbox.x) * scale;
        let feet = ((ux - 40.0) / 12.0).round();
        40.0 + feet * 12.0
    };

    // Press at A, drag to B (well away), release at B — the node drops at B.
    let (ax, ay) = (bbox.x + 120.0, bbox.y + 200.0);
    let (bx, by) = (bbox.x + 380.0, bbox.y + 200.0);
    let mouse = page.mouse();
    mouse.move_to(ax as i32, ay as i32, None).await.context("aim at A")?;
    mouse.down(None).await.context("press")?;
    mouse.move_to(bx as i32, by as i32, None).await.context("drag to B")?;
    mouse.up(None).await.context("release at B")?;

    let marker = page.locator("[data-testid='yard'] .house-corner").await;
    expect(marker.clone()).to_have_count(1).await.context("exactly one node dropped")?;
    let cx: f64 = marker
        .get_attribute("cx")
        .await
        .context("read the node cx")?
        .context("node has a cx")?
        .parse()
        .context("parse cx")?;

    assert!(
        (cx - snapped_cx(bx)).abs() < 0.5,
        "the node dropped at the release point B (cx={cx}, expected {})",
        snapped_cx(bx)
    );
    assert!(
        (cx - snapped_cx(ax)).abs() > 1.0,
        "the node did NOT drop at the press point A (cx={cx}, A maps to {})",
        snapped_cx(ax)
    );

    browser.close().await.context("close browser")?;
    Ok(())
}
