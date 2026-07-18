//! A1.1 e2e: editing a yard dimension reflows the canvas to the new scale.
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use anyhow::{Context, Result};
use common::{dist_dir, serve};
use playwright_rs::expect;
use playwright_rs::protocol::Playwright;

#[tokio::test]
async fn editing_width_reflows_the_grid() -> Result<()> {
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

    // Default 70×30 yard: 71 vertical + 31 horizontal grid lines + 1 scale bar.
    expect(page.locator("[data-testid='yard'] line"))
        .to_have_count(103)
        .await
        .context("grid at the default width")?;

    // Shrink the width to 20 ft; the canvas re-renders (21 vertical + 31
    // horizontal + 1 scale bar = 53 lines). `to_have_count` auto-retries, so it
    // waits for the reactive re-render.
    page.locator("[data-testid='yard-width']")
        .fill("20", None)
        .await
        .context("set width to 20 ft")?;
    expect(page.locator("[data-testid='yard'] line"))
        .to_have_count(53)
        .await
        .context("the grid reflows when the width changes")?;

    browser.close().await.context("close browser")?;
    Ok(())
}
