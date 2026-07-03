//! E1.5 e2e: the legend renders along the bottom strip, to the right of the
//! scale bar, with one entry per plan visual convention.
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
use playwright_rs::{BoundingBox, expect};

#[tokio::test]
async fn legend_renders_to_the_right_of_the_scale_bar() -> Result<()> {
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

    let legend = page.locator("[data-testid='legend']").await;
    expect(legend.clone())
        .to_be_visible()
        .await
        .context("the legend renders")?;

    for testid in [
        "legend-item-house",
        "legend-item-deck",
        "legend-item-planned",
        "legend-item-existing",
        "legend-item-planned-virtual",
        "legend-item-existing-virtual",
        "legend-item-selected",
        "legend-item-overflow",
    ] {
        expect(page.locator(&format!("[data-testid='{testid}']")).await)
            .to_have_count(1)
            .await
            .with_context(|| format!("missing legend entry: {testid}"))?;
    }

    // The legend's leftmost icon sits to the right of the scale bar's line.
    let scale_line = page.locator("[data-testid='yard'] line[stroke='#333']").await;
    let scale_box = scale_line
        .bounding_box()
        .await
        .context("measure the scale bar")?
        .context("scale bar has a bounding box")?;
    let legend_box = legend
        .bounding_box()
        .await
        .context("measure the legend")?
        .context("legend has a bounding box")?;
    let BoundingBox { x: scale_x, width: scale_w, .. } = scale_box;
    let BoundingBox { x: legend_x, .. } = legend_box;
    assert!(
        legend_x >= scale_x + scale_w,
        "legend ({legend_x}) starts to the right of the scale bar's end ({})",
        scale_x + scale_w
    );

    browser.close().await.context("close browser")?;
    Ok(())
}
