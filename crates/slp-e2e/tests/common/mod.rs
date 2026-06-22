//! Shared helpers for the slp-app browser e2e tests. (A `tests/common/` module is
//! compiled into each test binary, not as its own test target.)

use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::{Context, Result};
use axum::Router;
use tower_http::services::ServeDir;

/// Path to the Trunk-built `slp-app` dist directory.
pub fn dist_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../slp-app/dist")
}

/// Serve `dist` on an ephemeral local port; returns the address and the server
/// task handle (dropped when the test ends).
pub async fn serve(dist: &PathBuf) -> Result<(SocketAddr, tokio::task::JoinHandle<()>)> {
    let app = Router::new().fallback_service(ServeDir::new(dist));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .context("bind app server")?;
    let addr = listener.local_addr().context("local addr")?;
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve app");
    });
    Ok((addr, handle))
}
