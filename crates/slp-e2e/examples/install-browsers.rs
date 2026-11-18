//! Install the browser the bundled Playwright driver expects.
//!
//! Run before the e2e suite:
//! `cargo run --manifest-path crates/slp-e2e/Cargo.toml --example install-browsers`.
//!
//! The revision comes from playwright-rs itself, through `Cargo.lock`, so a
//! Dependabot bump of playwright-rs moves the browser with it. Naming a
//! Playwright version in the workflow instead (`npx playwright@x.y.z
//! install`) is a second version that nothing moves: a bump to a new driver
//! leaves CI installing the old chromium revision, and every test dies at
//! launch with "Executable doesn't exist".
//!
//! By default only the browser is installed. `-- --with-deps` also installs
//! the system libraries it needs, which the driver does through the system
//! package manager under `sudo` on Linux, and on Windows too; on macOS the
//! flag does nothing. CI passes it, since a fresh runner lacks the libraries.
//! A local run should not need it, and on a machine without apt or sudo it
//! fails before any browser is downloaded.

use anyhow::{Context, Result, bail};

#[tokio::main]
async fn main() -> Result<()> {
    let mut with_deps = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--with-deps" => with_deps = true,
            other => bail!("unknown argument `{other}`; the only one is --with-deps"),
        }
    }

    let browsers = Some(&["chromium"][..]);
    if with_deps {
        playwright_rs::install_browsers_with_deps(browsers).await
    } else {
        playwright_rs::install_browsers(browsers).await
    }
    .context("install chromium for the bundled Playwright driver")
}
