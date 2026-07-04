//! F2.2 e2e: while an item is armed, a shape-aware placement preview (a
//! translucent outline of the actual footprint — round for a round item)
//! tracks the pointer. Sanity-checks the ghost appears and follows the mouse;
//! dokime covers the shape/opacity details.
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use anyhow::Context;
use anyhow::Result;
use common::{arm_object, dist_dir, draw_central_deck, measure_ppf, serve};
use playwright_rs::BoundingBox;
use playwright_rs::protocol::Playwright;
use playwright_rs::expect;

#[tokio::test]
async fn the_preview_ghost_tracks_the_pointer_and_matches_the_armed_shape() -> Result<()> {
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

    let yard = page.locator("[data-testid='yard']").await;
    let ppf = measure_ppf(&yard).await?;
    draw_central_deck(&page, &yard, ppf).await?;
    let ppf = measure_ppf(&yard).await?;
    let BoundingBox { x, y, .. } = yard
        .bounding_box()
        .await
        .context("measure the yard")?
        .context("yard has a bounding box")?;

    // Before anything is armed, hovering shows the plain node marker, not a
    // shape preview.
    let mouse = page.mouse();
    mouse
        .move_to((x + 30.0 * ppf) as i32, (y + 15.0 * ppf) as i32, None)
        .await
        .context("hover with nothing armed")?;
    expect(page.locator("[data-testid='yard'] .placement-object-preview").await)
        .to_have_count(0)
        .await
        .context("no shape preview without an armed item")?;

    // Arm the (round) fire pit; hovering now shows a circular preview.
    arm_object(&page, "fire-pit").await?;
    mouse
        .move_to((x + 32.0 * ppf) as i32, (y + 15.0 * ppf) as i32, None)
        .await
        .context("hover the yard with the fire pit armed")?;
    let preview = page
        .locator("[data-testid='yard'] .placement-object-preview circle")
        .await;
    expect(preview.clone())
        .to_have_count(1)
        .await
        .context("the armed fire pit previews a circle")?;
    let cx1 = preview
        .get_attribute("cx")
        .await
        .context("read cx")?
        .context("circle has a cx")?;

    // Move elsewhere: the same preview circle follows.
    mouse
        .move_to((x + 45.0 * ppf) as i32, (y + 8.0 * ppf) as i32, None)
        .await
        .context("hover a different point")?;
    let cx2 = preview
        .get_attribute("cx")
        .await
        .context("read cx again")?
        .context("circle still has a cx")?;
    assert_ne!(cx1, cx2, "the preview tracks the pointer to a new position");

    browser.close().await.context("close browser")?;
    Ok(())
}
