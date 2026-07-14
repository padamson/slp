//! F3.0 e2e: draw a straight-edged area with the shared node-placement engine
//! and confirm it renders (with its area) and persists across a reload.
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
async fn draws_and_persists_a_shape() -> Result<()> {
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

    // Arm the area tool, drop four corners, snap-close.
    page.locator("[data-testid='draw-shape']")
        .await
        .click(None)
        .await
        .context("arm the shape tool")?;
    let yard = page.locator("[data-testid='yard']").await;
    let first = (140.0, 160.0);
    let corners = [first, (340.0, 160.0), (340.0, 300.0), (140.0, 300.0)];
    for (x, y) in corners {
        click_yard(&yard, x, y).await?;
    }
    click_yard(&yard, first.0, first.1).await?; // snap-close
    expect(page.locator("[data-testid='yard'] .shape-corner").await)
        .to_have_count(4)
        .await
        .context("the closed area has four corners")?;
    expect(page.locator("[data-testid='yard'] .shape polygon").await)
        .to_have_count(1)
        .await
        .context("the area footprint is drawn")?;
    expect(page.locator("[data-testid='yard'] .shape-label").await)
        .to_be_visible()
        .await
        .context("the area label (ft²) renders")?;

    // Reload — the area is restored from localStorage.
    page.reload(None).await.context("reload the page")?;
    expect(page.locator("[data-testid='yard'] .shape-corner").await)
        .to_have_count(4)
        .await
        .context("the area persists across a reload")?;

    browser.close().await.context("close browser")?;
    Ok(())
}
