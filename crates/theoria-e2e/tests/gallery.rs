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
