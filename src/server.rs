use anyhow::{Result, anyhow};
use axum::{
    Router,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::Response,
    routing::get,
};
use colored::Colorize;
use std::{path::PathBuf, sync::Arc};
use tokio::{fs, net::TcpListener};
use tracing::{debug, error, info, warn};

use crate::config::Config;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

const HTML_NOT_FOUND: &str = include_str!("../assets/not-found.html");
const HTML_INTERNAL_ERROR: &str = include_str!("../assets/internal-error.html");
const HTML_DEFAULT_INDEX: &str = include_str!("../assets/index-page.html");

#[derive(Clone)]
pub struct AppState {
    pages_dir: PathBuf,
    static_dir: PathBuf,
}

fn init_logging() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();
}

pub async fn start_server(config: &Config) -> Result<()> {
    println!(
        "\n {}{}",
        "\u{2F34B} Lime Web Server v".bright_green().bold(),
        env!("CARGO_PKG_VERSION").bright_green().bold()
    );
    if config.default {
        println!(
            "  {} {}",
            "".yellow().bold(),
            "In order to configure Lime, create 'lime.toml' file in the current directory.".bold()
        );
    }

    let listener = TcpListener::bind(&format!("{}:{}", config.host, config.port))
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    let pages_dir = PathBuf::from(&config.pages_dir);
    let static_dir = PathBuf::from(&config.static_dir);
    let state = Arc::new(AppState {
        pages_dir,
        static_dir,
    });

    let router = Router::new()
        .route("/", get(handle_index))
        .route("/{*path}", get(handle_wildcard))
        .with_state(state);

    init_logging();
    println!(
        "    {} http://{}:{}\n",
        "Available on:".bold(),
        config.host,
        config.port
    );
    axum::serve(listener, router)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    Ok(())
}

pub async fn handle_index(State(state): State<Arc<AppState>>) -> Response {
    let path = &state.pages_dir.join("index.html");
    if !path.exists() {
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body(Body::from(HTML_DEFAULT_INDEX))
            .unwrap()
    } else {
        serve_file(&state.pages_dir.join("index.html"), &state.pages_dir, true).await
    }
}

pub async fn handle_wildcard(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    info!(requested_path = %path, "Handling request");
    let extension = PathBuf::from(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("html")
        .to_lowercase();

    if extension.as_str() != "html" {
        debug!(path = %path, extension = %extension, "Serving static asset");
        serve_file(&state.static_dir.join(&path), &state.static_dir, false).await
    } else {
        debug!(path = %path, "Serving HTML file");
        serve_html(&path, &state.pages_dir).await
    }
}

async fn serve_html(path: &str, base_dir: &PathBuf) -> Response {
    let mut html_path = base_dir.join(path);
    if html_path.extension().is_none() {
        html_path.set_extension("html");
    }
    serve_file(&html_path, base_dir, true).await
}

async fn serve_file(file_path: &PathBuf, base_dir: &PathBuf, is_text: bool) -> Response {
    let base_canonical = match fs::canonicalize(base_dir).await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to canonicalize base dir: {}", e);
            return internal_error(base_dir).await;
        }
    };

    let full_canonical = match fs::canonicalize(file_path).await {
        Ok(p) => p,
        Err(_) => return not_found(base_dir).await,
    };

    if !full_canonical.starts_with(&base_canonical) {
        warn!("Path traversal attempt: {:?}", file_path);
        return not_found(base_dir).await;
    }

    let metadata = match fs::metadata(&full_canonical).await {
        Ok(m) => m,
        Err(_) => return not_found(base_dir).await,
    };

    if metadata.is_dir() {
        return not_found(base_dir).await;
    }

    let content = if is_text {
        match fs::read_to_string(&full_canonical).await {
            Ok(s) => s.into_bytes(),
            Err(e) => {
                error!("failed to read text file: {}", e);
                return internal_error(base_dir).await;
            }
        }
    } else {
        match fs::read(&full_canonical).await {
            Ok(b) => b,
            Err(e) => {
                error!("failed to read file: {}", e);
                return internal_error(base_dir).await;
            }
        }
    };

    let mime_type = mime_guess::from_path(&full_canonical)
        .first_or_octet_stream()
        .to_string();

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&mime_type)
            .unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", &mime_type)
        .body(Body::from(content))
        .unwrap()
}

async fn not_found(base_dir: &PathBuf) -> Response {
    let not_found_html = base_dir.join("not-found.html");

    if !not_found_html.exists() {
        return Response::builder()
            .header("Content-Type", "text/html")
            .status(StatusCode::NOT_FOUND)
            .body(Body::from(HTML_NOT_FOUND))
            .unwrap();
    }

    let content = match fs::read_to_string(&not_found_html).await {
        Ok(s) => s.into_bytes(),
        Err(e) => {
            error!("failed to read text file: {}", e);
            return internal_error(base_dir).await;
        }
    };

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Content-Type", "text/html")
        .body(Body::from(content))
        .unwrap()
}

async fn internal_error(base_dir: &PathBuf) -> Response {
    let not_found_html = base_dir.join("not-found.html");

    if !not_found_html.exists() {
        return default_internal_error().await;
    }

    let content = match fs::read_to_string(&not_found_html).await {
        Ok(s) => s.into_bytes(),
        Err(e) => {
            error!("failed to read text file: {}", e);
            return default_internal_error().await;
        }
    };

    return Response::builder()
        .header("Content-Type", "text/html")
        .status(StatusCode::NOT_FOUND)
        .body(Body::from(content))
        .unwrap();
}

async fn default_internal_error() -> Response {
    Response::builder()
        .header("Content-Type", "text/html")
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(HTML_INTERNAL_ERROR))
        .unwrap()
}
