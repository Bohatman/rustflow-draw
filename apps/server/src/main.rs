use rustflow_server::{run_server, ServerConfig};
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(6129);
    let static_dir = std::env::var("BOHAT_STATIC_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("apps/studio/dist"));

    let config = ServerConfig {
        host: "0.0.0.0".to_string(),
        port,
        static_dir,
    };
    if let Err(err) = run_server(config).await {
        eprintln!("server error: {err}");
        std::process::exit(1);
    }
}
