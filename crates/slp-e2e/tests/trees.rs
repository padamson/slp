//! e2e: place a tree from the palette (available immediately — no deck
//! required), see its canopy+trunk render and adjust their size, and see the
//! trunk flag red when it's standing on hardscape (deck/house) instead of open
//! ground.
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use anyhow::{Context, Result};
use common::{click_ft, dist_dir, draw_central_deck, measure_ppf, place_object, serve};
use playwright_rs::protocol::Playwright;
use playwright_rs::expect;

#[tokio::test]
async fn placing_a_tree_shows_canopy_and_trunk_with_no_rotate_handle() -> Result<()> {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!("skipping: {} not built (run `trunk build`).", dist.display());
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

    // Trees get their own palette group, for free (the palette groups by
    // category) — and it's available without drawing a deck first.
    expect(page.locator("[data-testid='palette-oak-tree']").await)
        .to_be_visible()
        .await
        .context("the oak tree tile is in the palette immediately")?;

    // Place it well off any structure (no deck/house drawn) — bare yard.
    place_object(&page, &yard, ppf, "oak-tree", 10.0, 15.0).await?;
    expect(page.locator("[data-testid='yard'] .furniture-item").await)
        .to_have_count(1)
        .await
        .context("the tree is on the plan")?;
    // Canopy + trunk are both circles.
    expect(page.locator("[data-testid='yard'] .furniture-item circle").await)
        .to_have_count(2)
        .await
        .context("a tree renders a canopy and a trunk")?;
    expect(page.locator("[data-testid='tree-trunk']").await)
        .to_have_count(1)
        .await
        .context("the trunk carries its own hook")?;
    expect(page.locator("[data-testid='estimate-total']").await)
        .to_be_visible()
        .await
        .context("the estimate reflects the tree")?;

    // Selecting the tree shows its diameter and canopy/trunk size inputs, but
    // no rotate handle (rotating a circle is a visual no-op).
    click_ft(&yard, ppf, 10.0, 15.0).await?;
    expect(page.locator("[data-testid='object-inspector']:has-text('⌀ 20 ft')").await)
        .to_have_count(1)
        .await
        .context("the inspector shows the tree's canopy diameter")?;
    expect(page.locator("[data-testid='canopy-diameter']").await)
        .to_be_visible()
        .await
        .context("a canopy-size input for a tree")?;
    expect(page.locator("[data-testid='trunk-diameter']").await)
        .to_be_visible()
        .await
        .context("a trunk-size input for a tree")?;
    expect(page.locator("[data-testid='rotate-handle']").await)
        .to_have_count(0)
        .await
        .context("a selected round item shows no rotate handle")?;

    // Bumping the canopy size grows the rendered circle.
    let canopy_input = page.locator("[data-testid='canopy-diameter']").await;
    canopy_input.fill("30", None).await.context("set canopy Ø to 30 ft")?;
    expect(page.locator("[data-testid='yard'] .furniture-item circle[r='180']").await)
        .to_have_count(1)
        .await
        .context("30 ft canopy Ø at 12 px/ft → r=180")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn dragging_the_canopy_or_trunk_handle_resizes_it() -> Result<()> {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!("skipping: {} not built (run `trunk build`).", dist.display());
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

    // A ⌀20 ft canopy / ⌀2 ft trunk oak tree at (30,15) — the canopy handle
    // starts at world (40,15), the trunk handle at (31,15).
    place_object(&page, &yard, ppf, "oak-tree", 30.0, 15.0).await?;
    click_ft(&yard, ppf, 30.0, 15.0).await?;

    // Drag the canopy handle out to (45,15) -> radius 15 -> Ø 30 ft.
    let canopy_handle = page.locator("[data-testid='canopy-handle']").await;
    common::drag_to_ft(&canopy_handle, &yard, ppf, 45.0, 15.0).await?;
    // Grew from Ø20 toward Ø30 — pixel-rounded to the nearest tenth of a foot,
    // not exactly 30.
    expect(page.locator("[data-testid='object-inspector']:has-text('⌀ 29.9 ft')").await)
        .to_have_count(1)
        .await
        .context("the canopy grew toward 30 ft")?;

    // Drag the trunk handle in to (30.5,15) -> radius 0.5 -> Ø 1 ft.
    let trunk_handle = page.locator("[data-testid='trunk-handle']").await;
    common::drag_to_ft(&trunk_handle, &yard, ppf, 30.5, 15.0).await?;
    expect(page.locator("[data-testid='trunk-diameter']").await)
        .to_have_value("1")
        .await
        .context("the trunk shrank to 1 ft")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn a_tree_trunk_flags_red_only_on_hardscape() -> Result<()> {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!("skipping: {} not built (run `trunk build`).", dist.display());
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
    draw_central_deck(&page, &yard, ppf).await?; // deck spans x:[28,42], y:[12,18]
    let ppf = measure_ppf(&yard).await?;

    // Place the tree on bare yard, well off the deck.
    place_object(&page, &yard, ppf, "oak-tree", 10.0, 15.0).await?;
    let tree = page.locator("[data-testid='yard'] .furniture-item").await;
    expect(page.locator("[data-testid='yard'] .furniture-item--trunk-invalid").await)
        .to_have_count(0)
        .await
        .context("a trunk on bare yard is fine")?;

    // Drag it onto the deck — the trunk should flag red.
    common::drag_to_ft(&tree, &yard, ppf, 35.0, 15.0).await?;
    expect(page.locator("[data-testid='yard'] .furniture-item--trunk-invalid").await)
        .to_have_count(1)
        .await
        .context("the trunk flags on the deck")?;

    // Drag it back off the deck — it should clear.
    common::drag_to_ft(&tree, &yard, ppf, 10.0, 15.0).await?;
    expect(page.locator("[data-testid='yard'] .furniture-item--trunk-invalid").await)
        .to_have_count(0)
        .await
        .context("the trunk clears back on bare yard")?;

    browser.close().await.context("close browser")?;
    Ok(())
}
