//! M4.4 e2e: give the Pavers material a photo, draw a paver area and a round
//! paver bed, and confirm both drawn surfaces fill with an SVG `<pattern>`
//! tiled at real-world scale — each fill becomes `url(#…-mat-paver)` and one
//! pattern per component references the material image, instead of the flat
//! paver gray.
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use anyhow::{Context, Result};
use common::{TRANSPARENT_PNG_1X1, click_ft, dist_dir, measure_ppf, serve, wait_attr};
use playwright_rs::expect;
use playwright_rs::protocol::Playwright;

#[tokio::test]
async fn a_material_photo_tiles_the_drawn_surfaces() -> Result<()> {
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
    let page = browser.new_page().await.context("new page")?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;

    // Give the Pavers material a photo via the catalog inspector.
    page.locator("[data-testid='edit-catalog']")
        .await
        .click(None)
        .await
        .context("open the catalog inspector")?;
    page.locator("[data-testid='catalog-row-paver']")
        .await
        .click(None)
        .await
        .context("select the paver material")?;
    page.locator("[data-testid='catalog-image']")
        .await
        .fill(TRANSPARENT_PNG_1X1, None)
        .await
        .context("set the material image")?;
    page.locator("[data-testid='catalog-close']")
        .await
        .click(None)
        .await
        .context("close the catalog inspector")?;

    let yard = page.locator("[data-testid='yard']").await;
    let ppf = measure_ppf(&yard).await?;

    // Arm the Pavers material + area tool, then draw a 10×8 ft area.
    page.locator("[data-testid='area-mat-paver']")
        .await
        .click(None)
        .await
        .context("arm the Pavers material")?;
    page.locator("[data-testid='draw-shape']")
        .await
        .click(None)
        .await
        .context("arm the area tool")?;
    let corners = [(10.0, 10.0), (20.0, 10.0), (20.0, 18.0), (10.0, 18.0)];
    for (fx, fy) in corners {
        click_ft(&yard, ppf, fx, fy).await?;
    }
    click_ft(&yard, ppf, corners[0].0, corners[0].1).await?; // snap-close

    // The area's polygon is filled by the material's pattern, not the flat gray.
    let poly = page.locator("[data-testid='yard'] .shape polygon").await;
    expect(poly.clone())
        .to_have_count(1)
        .await
        .context("the paver area is drawn")?;
    wait_attr(&poly, "fill", |f| f == "url(#area-mat-paver)")
        .await
        .context("the polygon is filled by its material's pattern")?;

    // A round paver bed tiles with the same photo through its own pattern.
    page.locator("[data-testid='draw-circle']")
        .await
        .click(None)
        .await
        .context("arm the circle tool")?;
    click_ft(&yard, ppf, 40.0, 15.0).await?; // center
    click_ft(&yard, ppf, 44.0, 15.0).await?; // rim (r = 4 ft)
    let disk = page
        .locator("[data-testid='yard'] .circle-area circle")
        .await;
    expect(disk.clone())
        .to_have_count(1)
        .await
        .context("the round paver bed is drawn")?;
    wait_attr(&disk, "fill", |f| f == "url(#circle-mat-paver)")
        .await
        .context("the disk is filled by its material's pattern")?;

    // One pattern per component (both referencing the same photo) — not one
    // per drawn area.
    let patterns = page.locator("[data-testid='yard'] pattern").await;
    expect(patterns)
        .to_have_count(2)
        .await
        .context("one shared pattern per component tiles the surfaces")?;
    let image_href = page
        .locator("[data-testid='yard'] pattern image")
        .await
        .first()
        .get_attribute("href")
        .await?
        .unwrap_or_default();
    assert_eq!(
        image_href, TRANSPARENT_PNG_1X1,
        "the pattern tiles the material photo"
    );

    browser.close().await.context("close browser")?;
    Ok(())
}
