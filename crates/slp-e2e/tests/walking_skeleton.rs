//! Slice 0 dogfood gate: serve the Trunk-built `slp-app` and drive it with
//! playwright-rs, asserting the WASM bundle boots and the yard renders to scale.
//! Because the app is a Leptos CSR/WASM app, this proves the bundle actually
//! runs — a static-HTML check could not. Feature behavior lives in sibling test
//! files (e.g. `yard_dimensions.rs`).
//!
//! Run after building the app:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent. Requires browsers:
//!   npx playwright@1.60.0 install chromium

mod common;

use anyhow::{Context, Result, ensure};
use common::{dist_dir, serve};
use playwright_rs::expect;
use playwright_rs::protocol::Playwright;

#[tokio::test]
async fn walking_skeleton_boots_and_renders_yard() -> Result<()> {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!(
            "skipping dogfood test: {} not built. Run `trunk build` in crates/slp-app first.",
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

    // The heading proves the WASM app mounted and painted (the body is empty
    // until the bundle boots and hydrates). The locator auto-waits — no sleep.
    expect(page.locator("h1").await)
        .to_have_text("Simple Landscape Planner")
        .await
        .context("heading renders once the WASM app boots")?;

    // The yard SVG is the canvas every later slice draws on; assert it mounted.
    expect(page.locator("[data-testid='yard']").await)
        .to_be_visible()
        .await
        .context("the yard SVG canvas is rendered")?;

    // Desired walking-skeleton behavior: the yard is drawn *to scale*.
    // The scale bar proves the foot→pixel rendering.
    expect(page.get_by_text("10 ft", false).await)
        .to_be_visible()
        .await
        .context("the scale bar renders (yard is drawn to scale)")?;

    // The ground rect is present... (`.ground`, not just `rect` — the legend's
    // icons are `<rect>`s too, so an unqualified selector is ambiguous).
    expect(page.locator("[data-testid='yard'] rect.ground").await)
        .to_be_visible()
        .await
        .context("the ground rect is rendered")?;

    // ...and the foot grid is drawn (many <line> elements; exact count tracks
    // yard dimensions, so assert a sane lower bound rather than a brittle exact).
    let grid_lines = page
        .locator("[data-testid='yard'] line")
        .await
        .count()
        .await
        .context("count grid lines")?;
    ensure!(grid_lines > 50, "the foot grid is rendered (got {grid_lines} lines)");

    browser.close().await.context("close browser")?;
    Ok(())
}
