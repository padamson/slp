//! R4 e2e: the photorealistic preview — the Overhead/Eye-level toggle, what the
//! app hands the render bridge, and the modal's success and failure states.
//!
//! The bridge is **stubbed**, so no image model is involved: these assert the
//! app's half of the contract (a real plan raster for overhead, none for
//! eye-level, the scene prompt built from what's drawn) and that the modal
//! reports what happened. The real `planRaster` still runs — it's the app's own
//! code — so this also covers rasterizing the live SVG.
//!
//! Build the app first, then run:
//!   (cd crates/slp-app && trunk build)
//!   cargo test --manifest-path crates/slp-e2e/Cargo.toml
//!
//! Skips gracefully when `crates/slp-app/dist` is absent.

mod common;

use anyhow::{Context, Result};
use common::{dist_dir, draw_central_deck, measure_ppf, place_object, serve};
use playwright_rs::expect;
use playwright_rs::protocol::{FilePayload, Page, Playwright};

/// Stub `window.slpRender.generate` to record its arguments and return a known
/// image, leaving the app's real `planRaster` in place.
const STUB_OK: &str = r#"(() => {
  const real = window.slpRender;
  window.slpRender = {
    planRaster: (...a) => real.planRaster(...a),
    generate: async (mode, prompt, controlImage, refsJson, configJson) => {
      window.__preview = { mode, prompt, controlImage, refsJson, configJson };
      return "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
    },
  };
  return "ok";
})()"#;

/// Stub the bridge as an unreachable backend.
const STUB_FAIL: &str = r#"(() => {
  const real = window.slpRender;
  window.slpRender = {
    planRaster: (...a) => real.planRaster(...a),
    generate: async () => {
      throw new Error("Can't reach the preview backend at http://localhost:7801 — is SwarmUI running?");
    },
  };
  return "ok";
})()"#;

async fn boot(page: &Page, addr: &std::net::SocketAddr, stub: &str) -> Result<()> {
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to app")?;
    page.evaluate_value(stub)
        .await
        .context("stub the render bridge")?;
    Ok(())
}

#[tokio::test]
async fn overhead_sends_a_plan_raster_and_shows_the_result() -> Result<()> {
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
    boot(&page, &addr, STUB_OK).await?;

    // Draw something so the prompt and the raster have content.
    let yard = page.locator("[data-testid='yard']");
    let ppf = measure_ppf(&yard).await?;
    draw_central_deck(&page, &yard, ppf).await?;
    let ppf = measure_ppf(&yard).await?;
    place_object(&page, &yard, ppf, "hot-tub-round", 10.0, 24.0).await?;

    // Overhead is the default mode.
    expect(page.locator("[data-testid='preview-mode-overhead'].active"))
        .to_have_count(1)
        .await
        .context("overhead is selected by default")?;

    page.locator("[data-testid='preview-generate']")
        .click(None)
        .await
        .context("click Preview")?;

    expect(page.locator("[data-testid='preview-image']"))
        .to_have_count(1)
        .await
        .context("the rendered image is shown")?;

    // What the app handed the bridge.
    let mode = page
        .evaluate_value("window.__preview.mode")
        .await
        .context("read the mode")?;
    assert_eq!(mode, "overhead");

    let has_raster = page
        .evaluate_value(
            "String((window.__preview.controlImage || '').startsWith('data:image/png;base64,'))",
        )
        .await?;
    assert_eq!(
        has_raster, "true",
        "overhead conditions on a real rasterized plan"
    );
    // The raster is a genuine render, not an empty canvas.
    let raster_len = page
        .evaluate_value("String((window.__preview.controlImage || '').length)")
        .await?;
    assert!(
        raster_len.parse::<usize>().unwrap_or(0) > 1000,
        "the raster has real content, got {raster_len} chars"
    );

    // The prompt is built from what's actually drawn.
    let prompt = page.evaluate_value("window.__preview.prompt").await?;
    assert!(
        prompt.contains("backyard") && prompt.contains("hot tub"),
        "the scene prompt describes the plan: {prompt}"
    );

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn eye_level_sends_the_prompt_without_a_raster() -> Result<()> {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!("skipping: {} not built.", dist.display());
        return Ok(());
    }
    let (addr, _server) = serve(&dist).await?;
    let pw = Playwright::launch().await.context("launch playwright")?;
    let browser = pw.chromium().launch().await.context("launch chromium")?;
    let page = common::new_page(&browser).await?;
    boot(&page, &addr, STUB_OK).await?;

    page.locator("[data-testid='preview-mode-eye-level']")
        .click(None)
        .await
        .context("switch to eye-level")?;
    expect(page.locator("[data-testid='preview-mode-eye-level'].active"))
        .to_have_count(1)
        .await
        .context("eye-level is now selected")?;

    page.locator("[data-testid='preview-generate']")
        .click(None)
        .await?;
    expect(page.locator("[data-testid='preview-image']"))
        .to_have_count(1)
        .await
        .context("an image comes back")?;

    let mode = page.evaluate_value("window.__preview.mode").await?;
    assert_eq!(mode, "eye-level");
    let control = page
        .evaluate_value("String(window.__preview.controlImage || '')")
        .await?;
    assert!(
        control.is_empty(),
        "eye-level sends no init image, got {} chars",
        control.len()
    );

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn an_unreachable_backend_is_reported_not_swallowed() -> Result<()> {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!("skipping: {} not built.", dist.display());
        return Ok(());
    }
    let (addr, _server) = serve(&dist).await?;
    let pw = Playwright::launch().await.context("launch playwright")?;
    let browser = pw.chromium().launch().await.context("launch chromium")?;
    let page = common::new_page(&browser).await?;
    boot(&page, &addr, STUB_FAIL).await?;

    page.locator("[data-testid='preview-generate']")
        .click(None)
        .await?;

    let failed = page.locator("[data-testid='preview-failed']");
    expect(failed.clone())
        .to_have_count(1)
        .await
        .context("the failure is surfaced")?;
    expect(failed)
        .to_contain_text("Can't reach the preview backend")
        .await
        .context("the message names the cause")?;
    expect(page.locator("[data-testid='preview-image']"))
        .to_have_count(0)
        .await
        .context("no image is shown behind a failure")?;

    // Dismissing returns to the plan.
    page.locator("[data-testid='preview-close']")
        .click(None)
        .await?;
    expect(page.locator("[data-testid='preview-modal']"))
        .to_have_count(0)
        .await
        .context("the modal closes")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

#[tokio::test]
async fn from_photo_conditions_on_the_uploaded_yard_photo() -> Result<()> {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!("skipping: {} not built.", dist.display());
        return Ok(());
    }
    let (addr, _server) = serve(&dist).await?;
    let pw = Playwright::launch().await.context("launch playwright")?;
    let browser = pw.chromium().launch().await.context("launch chromium")?;
    let page = common::new_page(&browser).await?;
    boot(&page, &addr, STUB_OK).await?;

    page.locator("[data-testid='preview-mode-from-photo']")
        .click(None)
        .await
        .context("switch to From photo")?;

    // With no photo yet there's nothing to condition on, so it can't generate.
    let generate = page.locator("[data-testid='preview-generate']");
    assert!(
        generate.get_attribute("disabled").await?.is_some(),
        "generating is blocked until a photo is chosen"
    );

    // Upload a yard photo (a tiny PNG stands in for a real one).
    page.locator("[data-testid='preview-photo']")
        .set_input_files_payload(
            FilePayload::new("yard.png", "image/png", b"stand-in-yard-photo".to_vec()),
            None,
        )
        .await
        .context("attach the yard photo")?;

    expect(page.locator("[data-testid='preview-photo-thumb']"))
        .to_have_count(1)
        .await
        .context("the chosen photo is shown back")?;

    page.locator("[data-testid='preview-generate']")
        .click(None)
        .await?;
    expect(page.locator("[data-testid='preview-image']"))
        .to_have_count(1)
        .await
        .context("a render comes back")?;

    // The photo — not the plan raster — is what conditioned it.
    let mode = page.evaluate_value("window.__preview.mode").await?;
    assert_eq!(mode, "from-photo");
    let is_photo = page
        .evaluate_value(
            "String((window.__preview.controlImage || '').startsWith('data:image/png'))",
        )
        .await?;
    assert_eq!(is_photo, "true", "the uploaded photo is the init image");

    // Removing it blocks generating again.
    page.locator("[data-testid='preview-close']")
        .click(None)
        .await?;
    page.locator("[data-testid='preview-photo-clear']")
        .click(None)
        .await
        .context("clear the photo")?;
    expect(page.locator("[data-testid='preview-photo-thumb']"))
        .to_have_count(0)
        .await
        .context("the thumbnail is gone")?;

    browser.close().await.context("close browser")?;
    Ok(())
}
