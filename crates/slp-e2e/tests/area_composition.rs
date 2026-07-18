//! B3.4 e2e: a paver area's composition editor. A drawn paver seeds its sub-base
//! courses (gravel base + bedding sand); selecting it opens a per-course editor
//! where changing a course's thickness or material — or adding/removing a layer
//! — reprices the estimate live, per area.
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use common::{click_ft, dist_dir, measure_ppf, serve};
use playwright_rs::protocol::Playwright;
use playwright_rs::{Locator, expect};

/// Poll a locator's `text_content` until it contains `needle` or times out.
async fn wait_contains(loc: &Locator, needle: &str) -> Result<String> {
    let start = Instant::now();
    loop {
        let text = loc.text_content().await?.unwrap_or_default();
        if text.contains(needle) {
            return Ok(text);
        }
        if start.elapsed() >= Duration::from_secs(5) {
            return Err(anyhow!("'{needle}' never appeared; last was '{text}'"));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// Poll until `loc`'s text does NOT contain `needle`, or time out.
async fn wait_absent(loc: &Locator, needle: &str) -> Result<()> {
    let start = Instant::now();
    loop {
        let text = loc.text_content().await?.unwrap_or_default();
        if !text.contains(needle) {
            return Ok(());
        }
        if start.elapsed() >= Duration::from_secs(5) {
            return Err(anyhow!("'{needle}' never went away; last was '{text}'"));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test]
async fn edits_a_paver_areas_composition() -> Result<()> {
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

    let yard = page.locator("[data-testid='yard']");
    let ppf = measure_ppf(&yard).await?;

    // Arm Pavers, draw a 10×8 ft patio (80 ft²), then click it to select.
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
    click_ft(&yard, ppf, 12.0, 11.0).await?; // select the patio

    // The composition editor opens, seeded with the paver's two courses.
    expect(page.locator("[data-testid='course-editor']"))
        .to_be_visible()
        .await
        .context("selecting the paver opens its composition editor")?;
    let rows = page.locator("[data-testid^='course-row-']");
    expect(rows.clone())
        .to_have_count(2)
        .await
        .context("seeded with a base + bedding course")?;

    // Only aggregates are offered as courses — mulch (a surface bed) is not.
    let editor = page.locator("[data-testid='course-editor']");
    let editor_text = editor.text_content().await?.unwrap_or_default();
    assert!(
        editor_text.contains("Gravel base"),
        "gravel is offered: {editor_text}"
    );
    assert!(
        !editor_text.contains("Mulch"),
        "mulch is never a paver course: {editor_text}"
    );

    // The estimate lists the gravel base at its seeded 4 in (80·4/324 ≈ 1.0 yd³).
    let estimate = page.locator("[data-testid='estimate']");
    wait_contains(&estimate, "Gravel base")
        .await
        .context("the estimate itemizes the gravel base")?;

    // Deepen the base course (row 0) to 6 in — the gravel volume grows live
    // (80·6/324 ≈ 1.5 yd³).
    page.locator("[data-testid='course-row-0'] input")
        .fill("6", None)
        .await
        .context("set the base course to 6 in")?;
    wait_contains(&estimate, "1.5 yd³")
        .await
        .context("the estimate reprices the deeper gravel base")?;

    // Swap the base course's material to bedding sand — now no course is gravel,
    // so the Gravel base line leaves the estimate (per-area material choice).
    page.locator("[data-testid='course-row-0'] select")
        .select_option("paver-sand", None)
        .await
        .context("change the base course material to sand")?;
    wait_absent(&estimate, "Gravel base")
        .await
        .context("the gravel line drops once no course uses it")?;

    // Remove the bedding course (row 1) — down to one course.
    page.locator("[data-testid='course-remove-1']")
        .click(None)
        .await
        .context("remove the bedding course")?;
    expect(rows.clone())
        .to_have_count(1)
        .await
        .context("a course was removed")?;

    // Add a layer back — it defaults to a gravel aggregate, never mulch.
    page.locator("[data-testid='course-add']")
        .click(None)
        .await
        .context("add a layer")?;
    expect(rows)
        .to_have_count(2)
        .await
        .context("a course was added")?;
    assert_eq!(
        page.locator("[data-testid='course-row-1'] select")
            .input_value(None)
            .await?,
        "paver-base",
        "a new layer defaults to a gravel aggregate, not mulch"
    );

    browser.close().await.context("close browser")?;
    Ok(())
}
