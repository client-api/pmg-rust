//! Example: list cluster nodes.
//!
//! Run with:
//!
//! ```sh
//! PMG_HOST=https://pmg.example.com:8006 \
//! PMG_TOKEN='root@pam!auto=...' \
//! cargo run --example list_nodes
//! ```

use clientapi_pmg::apis::configuration::Configuration;
use clientapi_pmg::apis::nodes_api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cfg = Configuration::new();
    cfg.base_path = format!(
        "{}/api2/json",
        std::env::var("PMG_HOST").unwrap_or_else(|_| "https://localhost:8006".into()),
    );
    cfg.bearer_access_token = std::env::var("PMG_TOKEN").ok();

    let resp = nodes_api::nodes_get_nodes(&cfg).await?;
    // Non-PVE products: the upstream apidoc.js declares this endpoint
    // `returns: { type: null }`, so the generator emits `data` as an
    // untyped `serde_json::Value` (or similar). Print whatever came back.
    println!("Response: {:?}", resp);
    Ok(())
}
