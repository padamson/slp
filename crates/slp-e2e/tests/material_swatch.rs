//! M4.4 e2e: a material's swatch — the small square shown in the catalog list,
//! the area material picker, and the area inspector — becomes the material's
//! photo once one is set, in all three places (a flat category color before).
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
async fn a_material_photo_becomes_the_swatch_in_every_panel() -> Result<()> {
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

    // The area material picker shows a paver swatch — a flat color square
    // (no photo) before any image is set.
    let picker_swatch =
        page.locator("[data-testid='area-mat-cat-paver'] [data-testid='material-swatch']");
    expect(picker_swatch.clone())
        .to_have_count(1)
        .await
        .context("the picker shows a paver swatch")?;
    assert!(
        picker_swatch.get_attribute("src").await?.is_none(),
        "the swatch is a flat color square (no image src) before an image is set"
    );

    // Give the Pavers material a photo via the catalog inspector.
    page.locator("[data-testid='edit-catalog']")
        .click(None)
        .await
        .context("open the catalog inspector")?;
    page.locator("[data-testid='catalog-row-paver']")
        .click(None)
        .await
        .context("select the paver material")?;
    page.locator("[data-testid='catalog-image']")
        .fill(TRANSPARENT_PNG_1X1, None)
        .await
        .context("set the material image")?;

    // 1) The catalog list row's swatch is now the photo.
    let row_swatch =
        page.locator("[data-testid='catalog-row-paver'] [data-testid='material-swatch']");
    wait_attr(&row_swatch, "src", |s| s == TRANSPARENT_PNG_1X1)
        .await
        .context("the catalog row swatch becomes the photo")?;

    // 2) The area picker's swatch is now the photo (live, same signal).
    wait_attr(&picker_swatch, "src", |s| s == TRANSPARENT_PNG_1X1)
        .await
        .context("the picker swatch becomes the photo")?;

    page.locator("[data-testid='catalog-close']")
        .click(None)
        .await
        .context("close the catalog inspector")?;

    // 3) Draw a paver area and select it — the area inspector's swatch is the
    // photo too.
    let yard = page.locator("[data-testid='yard']");
    let ppf = measure_ppf(&yard).await?;
    page.locator("[data-testid='area-mat-cat-paver']")
        .click(None)
        .await
        .context("arm the Pavers material")?;
    page.locator("[data-testid='draw-shape']")
        .click(None)
        .await
        .context("arm the area tool")?;
    let corners = [(10.0, 10.0), (20.0, 10.0), (20.0, 18.0), (10.0, 18.0)];
    for (fx, fy) in corners {
        click_ft(&yard, ppf, fx, fy).await?;
    }
    click_ft(&yard, ppf, corners[0].0, corners[0].1).await?; // snap-close
    click_ft(&yard, ppf, 15.0, 14.0).await?; // click inside to select

    let inspector_swatch =
        page.locator("[data-testid='area-inspector'] [data-testid='material-swatch']");
    expect(inspector_swatch.clone())
        .to_have_count(1)
        .await
        .context("the area inspector shows a material swatch")?;
    wait_attr(&inspector_swatch, "src", |s| s == TRANSPARENT_PNG_1X1)
        .await
        .context("the area inspector swatch is the photo")?;

    browser.close().await.context("close browser")?;
    Ok(())
}
