//! e2e: select the house (or a deck level) by clicking its body — mirroring
//! how clicking a drawn area or a tree selects it — and drag one of its
//! corners to move it. A house corner move is checked against a door on that
//! same wall (its position is derived live from the wall + offset, so it
//! should keep rendering, not vanish or crash) — moving a deck level's corner
//! has no such dependent geometry to check.
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use anyhow::{Context, Result};
use common::{YARD_D, click_ft, dist_dir, measure_ppf, serve};
use playwright_rs::protocol::Playwright;
use playwright_rs::{BoundingBox, expect};

/// The app's fixed SVG user-space scale (`planner.rs::PX_FT`) — the `points`
/// attribute is always in these units, independent of the rendered/CSS scale
/// `measure_ppf` reports (which is for translating a world-feet point to a
/// real page-pixel click/drag position).
const SVG_PX_FT: f64 = 12.0;

#[tokio::test]
async fn selects_the_house_and_moves_a_corner() -> Result<()> {
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

    let yard = page.locator("[data-testid='yard']").await;
    let ppf = measure_ppf(&yard).await?;

    // Draw a 10x8 ft house.
    page.locator("[data-testid='draw-house']")
        .await
        .click(None)
        .await
        .context("arm the house tool")?;
    let corners = [(10.0, 10.0), (20.0, 10.0), (20.0, 18.0), (10.0, 18.0)];
    for (fx, fy) in corners {
        click_ft(&yard, ppf, fx, fy).await?;
    }
    click_ft(&yard, ppf, corners[0].0, corners[0].1).await?; // snap-close
    expect(page.locator("[data-testid='yard'] .house-corner").await)
        .to_have_count(4)
        .await
        .context("the house is drawn")?;

    // A door on the wall between corners 0 and 1 (the one whose endpoint is
    // about to move) — its position is derived live from the wall, so it's
    // the dependent geometry this test pins.
    page.locator("[data-testid='add-door']")
        .await
        .click(None)
        .await
        .context("arm door placement")?;
    click_ft(&yard, ppf, 13.0, 10.0).await?;
    click_ft(&yard, ppf, 15.0, 10.0).await?;
    expect(page.locator("[data-testid='yard'] .door").await)
        .to_have_count(1)
        .await
        .context("a door spans the wall")?;

    // Click inside the house body (off any corner/door) to select it — no
    // armed tool, mirroring how clicking a drawn area or a tree selects it.
    click_ft(&yard, ppf, 12.0, 12.0).await?;
    expect(page.locator("[data-testid='yard'] .house--selected").await)
        .to_have_count(1)
        .await
        .context("the house is selected")?;
    expect(page.locator("[data-testid='house-node']").await)
        .to_have_count(4)
        .await
        .context("its corners become interactive node handles")?;

    // Drag corner 0 (world (10,10), on the door's wall) up to (10,13).
    let BoundingBox { x, y, .. } = yard
        .bounding_box()
        .await
        .context("measure the yard")?
        .context("yard has a bounding box")?;
    let screen = |fx: f64, fy: f64| (x + fx * ppf, y + (YARD_D - fy) * ppf);
    let mouse = page.mouse();
    let node0 = page.locator("[data-testid='house-node']").await.nth(0);
    node0.hover(None).await.context("hover corner 0")?;
    mouse.down(None).await.context("press corner 0")?;
    let (mx, my) = screen(10.0, 13.0);
    mouse
        .move_to(mx as i32, my as i32, None)
        .await
        .context("drag it up")?;
    mouse.up(None).await.context("release")?;

    // The wall moved with the corner (its polygon should no longer contain
    // the original corner's screen point) and the door — deriving its
    // position from the wall's *current* geometry each render — still renders,
    // not vanished or crashed.
    let points = page
        .locator("[data-testid='yard'] .house polygon")
        .await
        .get_attribute("points")
        .await
        .context("read the house polygon points")?
        .context("polygon has a points attribute")?;
    // The `points` attribute is in SVG viewBox space (feet · SVG_PX_FT, y flipped),
    // not page pixels — a different unit than `screen`'s mouse coordinates.
    let svg_pt = |fx: f64, fy: f64| format!("{},{}", fx * SVG_PX_FT, (YARD_D - fy) * SVG_PX_FT);
    assert!(
        !points.contains(&svg_pt(10.0, 10.0)),
        "the moved corner no longer sits at its original point"
    );
    expect(page.locator("[data-testid='yard'] .door").await)
        .to_have_count(1)
        .await
        .context("the door on that wall still renders after the move")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn selects_a_deck_level_and_moves_a_corner() -> Result<()> {
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

    let yard = page.locator("[data-testid='yard']").await;
    let ppf = measure_ppf(&yard).await?;

    // Draw a 10x8 ft deck level (default elevation 1.0 ft).
    page.locator("[data-testid='draw-deck']")
        .await
        .click(None)
        .await
        .context("arm the deck tool")?;
    let corners = [(30.0, 10.0), (40.0, 10.0), (40.0, 18.0), (30.0, 18.0)];
    for (fx, fy) in corners {
        click_ft(&yard, ppf, fx, fy).await?;
    }
    click_ft(&yard, ppf, corners[0].0, corners[0].1).await?; // snap-close
    expect(page.locator("[data-testid='yard'] .deck-corner").await)
        .to_have_count(4)
        .await
        .context("the level is drawn")?;

    // Click inside the level's body (off any corner) to select it.
    click_ft(&yard, ppf, 32.0, 12.0).await?;
    expect(
        page.locator("[data-testid='yard'] .deck-level--selected")
            .await,
    )
    .to_have_count(1)
    .await
    .context("the level is selected")?;
    expect(page.locator("[data-testid='deck-node']").await)
        .to_have_count(4)
        .await
        .context("its corners become interactive node handles")?;

    // Drag corner 0 (world (30,10)) up to (30,13).
    let BoundingBox { x, y, .. } = yard
        .bounding_box()
        .await
        .context("measure the yard")?
        .context("yard has a bounding box")?;
    let screen = |fx: f64, fy: f64| (x + fx * ppf, y + (YARD_D - fy) * ppf);
    let mouse = page.mouse();
    let node0 = page.locator("[data-testid='deck-node']").await.nth(0);
    node0.hover(None).await.context("hover corner 0")?;
    mouse.down(None).await.context("press corner 0")?;
    let (mx, my) = screen(30.0, 13.0);
    mouse
        .move_to(mx as i32, my as i32, None)
        .await
        .context("drag it up")?;
    mouse.up(None).await.context("release")?;

    let points = page
        .locator("[data-testid='yard'] .deck-level polygon")
        .await
        .get_attribute("points")
        .await
        .context("read the level polygon points")?
        .context("polygon has a points attribute")?;
    // The `points` attribute is in SVG viewBox space (feet · SVG_PX_FT, y flipped),
    // not page pixels — a different unit than `screen`'s mouse coordinates.
    let svg_pt = |fx: f64, fy: f64| format!("{},{}", fx * SVG_PX_FT, (YARD_D - fy) * SVG_PX_FT);
    assert!(
        !points.contains(&svg_pt(30.0, 10.0)),
        "the moved corner no longer sits at its original point"
    );

    browser.close().await.context("close browser")?;
    Ok(())
}
