//! H1.2b e2e: after drawing a house, click "Add door" then click a wall to
//! place a door; confirm it renders and persists across a reload.
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
async fn places_a_door_on_a_wall_and_persists() -> Result<()> {
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

    // Draw a rectangular house: enter draw mode, drop four corners, snap-close.
    page.locator("[data-testid='draw-house']")
        .await
        .click(None)
        .await
        .context("enter draw mode")?;
    let yard = page.locator("[data-testid='yard']").await;
    let corners = [(120.0, 120.0), (360.0, 120.0), (360.0, 300.0), (120.0, 300.0)];
    for (x, y) in corners {
        click_yard(&yard, x, y).await?;
    }
    click_yard(&yard, 120.0, 120.0).await?; // snap-close
    expect(page.locator("[data-testid='yard'] .house-corner").await)
        .to_have_count(4)
        .await
        .context("the house is drawn")?;

    // Add a door on the top wall: click two points on it to span the opening.
    page.locator("[data-testid='add-door']")
        .await
        .click(None)
        .await
        .context("arm door placement")?;
    click_yard(&yard, 200.0, 120.0).await?;
    click_yard(&yard, 280.0, 120.0).await?;
    expect(page.locator("[data-testid='yard'] .door").await)
        .to_have_count(1)
        .await
        .context("a door spans the two clicked points on the wall")?;

    // Reload — the door is restored from localStorage.
    page.reload(None).await.context("reload the page")?;
    expect(page.locator("[data-testid='yard'] .door").await)
        .to_have_count(1)
        .await
        .context("the door persists across a reload")?;

    browser.close().await.context("close browser")?;
    Ok(())
}
