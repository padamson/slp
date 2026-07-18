//! A1.1 persistence e2e: the yard size is saved to localStorage and restored on
//! reload. This is browser-only behavior (localStorage + a full reload), so it
//! can't be a dokime test — under ssr the save/restore are no-ops.
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
async fn yard_size_persists_across_reload() -> Result<()> {
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

    // Change the width to 40 ft; the grid reflows to 41 vertical + 31 horizontal
    // + 1 scale bar = 73 lines (default 70 ft would be 103).
    page.locator("[data-testid='yard-width']")
        .fill("40", None)
        .await
        .context("set width to 40 ft")?;
    expect(page.locator("[data-testid='yard'] line"))
        .to_have_count(73)
        .await
        .context("grid reflows to the new width")?;

    // Reload — the Plan is restored from localStorage, so the grid is still 73
    // (it would reset to 103 if persistence were broken).
    page.reload(None).await.context("reload the page")?;
    expect(page.locator("[data-testid='yard'] line"))
        .to_have_count(73)
        .await
        .context("yard size persists across reload (restored from localStorage)")?;

    browser.close().await.context("close browser")?;
    Ok(())
}
