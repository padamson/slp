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
