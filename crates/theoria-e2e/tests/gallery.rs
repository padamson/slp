//! theoria's own browser e2e: serve a gallery built from theoria's
//! components-as-fixtures and drive it with playwright-rs.
//!
//! The core check is generic — like Storybook's test-runner, it discovers every
//! story from the gallery's own sidebar at runtime and asserts each one renders
//! and selects. Add a story (a `*.stories.rs`) and it's covered automatically;
//! no compile-time list here. This is theoria dogfooding itself one level — never
//! a `Gallery` inside a `Gallery`.
//!
//! Build the gallery first, then run:
//!   cargo run -p theoria-cli -- build --config theoria-e2e.toml
//!   cargo test --manifest-path crates/theoria-e2e/Cargo.toml
//!
//! Skips gracefully when the gallery dist is absent. Requires browsers:
//!   npx playwright@1.60.0 install chromium

use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use axum::Router;
use playwright_rs::expect;
use playwright_rs::protocol::{Page, Playwright};
use tower_http::services::ServeDir;

fn dist_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/theoria/gallery/theoria/dist")
}

async fn serve(dist: &PathBuf) -> Result<(SocketAddr, tokio::task::JoinHandle<()>)> {
    let app = Router::new().fallback_service(ServeDir::new(dist));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .context("bind gallery server")?;
    let addr = listener.local_addr().context("local addr")?;
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve gallery");
    });
    Ok((addr, handle))
}

/// Discover every story from the gallery sidebar and assert each one selects and
/// renders non-empty content on the stage. Generic over whatever stories the
/// gallery was built with.
async fn assert_every_story_renders(page: &Page) -> Result<()> {
    let names = page
        .locator(".theoria > .theoria-nav button")
        .await
        .all_inner_texts()
        .await
        .context("read story names from the sidebar")?;
    ensure!(!names.is_empty(), "gallery lists at least one story");

    for name in &names {
        page.get_by_text(name, true)
            .await
            .click(None)
            .await
            .with_context(|| format!("click story {name:?}"))?;

        let active = page
            .locator(".theoria > .theoria-nav button.active")
            .await
            .inner_text()
            .await
            .context("read active story")?;
        ensure!(
            active.trim() == name.trim(),
            "selecting {name:?} should mark it active, but active was {active:?}"
        );

        let stage_children = page
            .locator(".theoria-stage *")
            .await
            .count()
            .await
            .context("count stage content")?;
        ensure!(stage_children > 0, "story {name:?} renders non-empty content");
    }
    Ok(())
}

#[tokio::test]
async fn every_story_renders_and_selects() -> Result<()> {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!(
            "skipping theoria e2e: {} not built. Run\n  \
             cargo run -p theoria-cli -- build --config theoria-e2e.toml",
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
        .context("navigate to gallery")?;

    assert_every_story_renders(&page).await?;

    browser.close().await.context("close browser")?;
    Ok(())
}

/// The controls panel drives the stage live: editing a knob re-renders the
/// story view, and the show-code block carries the captured source. This is what
/// dokime's SSR snapshots can't prove — the interactive re-render.
#[tokio::test]
async fn controls_drive_the_stage_live() -> Result<()> {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!("skipping theoria e2e: {} not built.", dist.display());
        return Ok(());
    }

    let (addr, _server) = serve(&dist).await?;
    let pw = Playwright::launch().await.context("launch playwright")?;
    let browser = pw.chromium().launch().await.context("launch chromium")?;
    let page = browser.new_page().await.context("new page")?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to gallery")?;

    // Select the knobs demo (a `#[story]` with args → renders a controls panel).
    page.get_by_text("Knobs · demo", true)
        .await
        .click(None)
        .await
        .context("select the knobs demo")?;

    // Default `on = true` → the stage shows "ON".
    expect(page.locator(".theoria-stage .k-flag").await)
        .to_have_text("ON")
        .await
        .context("flag starts ON")?;

    // Toggle the bool control off; the stage re-renders to "OFF".
    page.locator(".theoria-panel input[type=checkbox]")
        .await
        .click(None)
        .await
        .context("toggle the flag control")?;
    expect(page.locator(".theoria-stage .k-flag").await)
        .to_have_text("OFF")
        .await
        .context("toggling the control re-renders the stage")?;

    // Show-code carries the captured source.
    expect(page.locator(".theoria-panel .theoria-code summary").await)
        .to_have_text("Show code")
        .await
        .context("show-code toggle present")?;

    browser.close().await.context("close browser")?;
    Ok(())
}

/// The selected story survives a full page reload (persisted to localStorage) —
/// this is what keeps you on your story across Trunk's hot reload.
#[tokio::test]
async fn selection_persists_across_reload() -> Result<()> {
    let dist = dist_dir();
    if !dist.join("index.html").exists() {
        eprintln!("skipping theoria e2e: {} not built.", dist.display());
        return Ok(());
    }

    let (addr, _server) = serve(&dist).await?;
    let pw = Playwright::launch().await.context("launch playwright")?;
    let browser = pw.chromium().launch().await.context("launch chromium")?;
    let page = browser.new_page().await.context("new page")?;
    page.goto(&format!("http://{addr}"), None)
        .await
        .context("navigate to gallery")?;

    // Select a non-default story...
    page.get_by_text("StoryNav · single", false)
        .await
        .click(None)
        .await
        .context("select the second story")?;
    expect(page.locator(".theoria > .theoria-nav button.active").await)
        .to_have_text("StoryNav · single")
        .await
        .context("second story is active before reload")?;

    // ...reload the page; the selection is restored from localStorage.
    page.reload(None).await.context("reload the page")?;
    expect(page.locator(".theoria > .theoria-nav button.active").await)
        .to_have_text("StoryNav · single")
        .await
        .context("selected story persists across reload")?;

    browser.close().await.context("close browser")?;
    Ok(())
}
