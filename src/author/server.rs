use axum::{
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use rust_embed::Embed;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Embed)]
#[folder = "src/web/"]
struct WebAssets;

/// Shared application state passed to all handlers.
#[derive(Clone)]
pub struct AppState {
    /// Currently loaded course path (None = welcome screen, waiting for user to open/create)
    pub course_path: Arc<RwLock<Option<PathBuf>>>,
    /// Detected author name for audit trail
    pub author_name: String,
}

/// Start the Course Designer web server.
pub fn start(course_path: Option<&Path>, port: u16, no_open: bool) -> anyhow::Result<()> {
    // If a path is provided, verify it exists
    if let Some(p) = course_path {
        let course_yaml = p.join("course.yaml");
        if !course_yaml.exists() {
            anyhow::bail!(
                "course.yaml not found in {}. Is this a course directory?",
                p.display()
            );
        }
    }

    let author_name = super::workspace::detect_author();

    let state = Arc::new(AppState {
        course_path: Arc::new(RwLock::new(course_path.map(|p| p.to_path_buf()))),
        author_name,
    });

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let app = Router::new()
            .nest("/api", super::api::routes())
            .route("/ws/preview", get(super::preview::ws_preview))
            .fallback(get(serve_static))
            .with_state(state);

        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = tokio::net::TcpListener::bind(addr).await?;
        let local_addr = listener.local_addr()?;
        let url = format!("http://{}", local_addr);

        println!("LearnLocal Studio running at {}", url);
        if let Some(p) = course_path {
            println!("Editing: {}", p.display());
        } else {
            println!("No course loaded — open the browser to create or open a project.");
        }
        println!("Press Ctrl+C to stop.\n");

        if !no_open {
            let _ = open::that(&url);
        }

        axum::serve(listener, app).await?;
        Ok::<(), anyhow::Error>(())
    })
}

async fn serve_static(uri: axum::http::Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match WebAssets::get(path) {
        Some(content) => {
            let mime = mime_from_path(path);
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime)],
                content.data.into_owned(),
            )
                .into_response()
        }
        None => match WebAssets::get("index.html") {
            Some(content) => {
                Html(String::from_utf8_lossy(&content.data).to_string()).into_response()
            }
            None => (StatusCode::NOT_FOUND, "Not found").into_response(),
        },
    }
}

fn mime_from_path(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("svg") => "image/svg+xml",
        Some("woff2") => "font/woff2",
        Some("map") => "application/json",
        _ => "application/octet-stream",
    }
}
