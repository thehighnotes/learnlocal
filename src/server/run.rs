use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::auth::TokenCache;
use super::db::Database;

pub struct ServerState {
    pub db: Database,
    pub packages_dir: PathBuf,
    pub github_client_id: Option<String>,
    #[allow(dead_code)]
    pub github_client_secret: Option<String>,
    pub token_cache: TokenCache,
}

pub fn start(port: u16, data_dir: &Path, packages_dir: &Path) -> anyhow::Result<()> {
    // Ensure directories exist
    std::fs::create_dir_all(data_dir)?;
    std::fs::create_dir_all(packages_dir)?;

    let db = Database::open(&data_dir.join("community.db"))?;
    db.init_schema()?;

    let state = Arc::new(ServerState {
        db,
        packages_dir: packages_dir.to_path_buf(),
        github_client_id: std::env::var("GITHUB_CLIENT_ID").ok(),
        github_client_secret: std::env::var("GITHUB_CLIENT_SECRET").ok(),
        token_cache: TokenCache::new(),
    });

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let app = axum::Router::new()
            .nest("/api/v1", super::api::routes())
            .route("/health", axum::routing::get(health))
            .with_state(state);

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr).await?;
        println!(
            "LearnLocal Community Server listening on http://{}",
            listener.local_addr()?
        );
        println!("  Data:     {}", data_dir.display());
        println!("  Packages: {}", packages_dir.display());
        axum::serve(listener, app).await?;
        Ok::<(), anyhow::Error>(())
    })
}

async fn health() -> &'static str {
    r#"{"status":"ok"}"#
}
