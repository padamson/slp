//! H2 e2e: draw the deck footprint with the shared node-placement engine and
//! confirm it persists across a reload.
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
async fn draws_and_persists_the_deck() -> Result<()> {
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

    // Arm the deck tool, drop four corners, snap-close.
    page.locator("[data-testid='draw-deck']")
        .click(None)
        .await
        .context("arm the deck tool")?;
    let yard = page.locator("[data-testid='yard']");
    let first = (140.0, 160.0);
    let corners = [first, (340.0, 160.0), (340.0, 300.0), (140.0, 300.0)];
    for (x, y) in corners {
        click_yard(&yard, x, y).await?;
    }
    click_yard(&yard, first.0, first.1).await?; // snap-close
    expect(page.locator("[data-testid='yard'] .deck-corner"))
        .to_have_count(4)
        .await
        .context("the closed deck has four corners")?;
    expect(page.locator("[data-testid='yard'] .deck polygon"))
        .to_have_count(1)
        .await
        .context("the deck footprint is drawn")?;

    // Reload — the deck is restored from localStorage.
    page.reload(None).await.context("reload the page")?;
    expect(page.locator("[data-testid='yard'] .deck-corner"))
        .to_have_count(4)
        .await
        .context("the deck persists across a reload")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn adds_steps_to_a_deck_edge_and_persists() -> Result<()> {
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

    // Draw a deck level (default elevation), then close it.
    page.locator("[data-testid='draw-deck']")
        .click(None)
        .await
        .context("arm the deck tool")?;
    let yard = page.locator("[data-testid='yard']");
    let corners = [
        (140.0, 160.0),
        (340.0, 160.0),
        (340.0, 300.0),
        (140.0, 300.0),
    ];
    for (x, y) in corners {
        click_yard(&yard, x, y).await?;
    }
    click_yard(&yard, 140.0, 160.0).await?; // snap-close
    expect(page.locator("[data-testid='yard'] .deck-corner"))
        .to_have_count(4)
        .await
        .context("the deck is drawn")?;

    // Add steps on the bottom edge: two clicks span the run.
    page.locator("[data-testid='add-steps']")
        .click(None)
        .await
        .context("arm the steps tool")?;
    click_yard(&yard, 190.0, 300.0).await?;
    click_yard(&yard, 290.0, 300.0).await?;
    expect(page.locator("[data-testid='yard'] .steps"))
        .to_have_count(1)
        .await
        .context("a step run is added to the deck edge")?;

    // Reload — the steps persist.
    page.reload(None).await.context("reload the page")?;
    expect(page.locator("[data-testid='yard'] .steps"))
        .to_have_count(1)
        .await
        .context("the steps persist across a reload")?;

    browser.close().await.context("close browser")?;
    Ok(())
}
