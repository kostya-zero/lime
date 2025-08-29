use anyhow::{Result, anyhow};
use axum::{
    Router,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse},
    routing::get,
};
use colored::Colorize;
use std::path::PathBuf;
use tokio::{fs, net::TcpListener};
use tracing::{debug, error, info, warn};

use crate::config::Config;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

const HTML_NOT_FOUND: &str = include_str!("../assets/not-found.html");
const HTML_INTERNAL_ERROR: &str = include_str!("../assets/internal-error.html");
const HTML_DEFAULT_INDEX: &str = include_str!("../assets/index-page.html");

#[derive(Clone)]
pub struct AppState {
    working_dir: String,
    static_dir: String,
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

    let state = AppState {
        working_dir: config.pages_dir.clone(),
        static_dir: config.static_dir.clone(),
    };

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

pub async fn handle_index(State(state): State<AppState>) -> impl IntoResponse {
    let path = PathBuf::from(state.working_dir).join("index.html");
    if !path.exists() {
        return (StatusCode::OK, Html(HTML_DEFAULT_INDEX)).into_response();
    }

    let file = fs::read_to_string(path).await;
    if file.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, Html(HTML_INTERNAL_ERROR)).into_response();
    }

    let content = file.unwrap();
    (StatusCode::OK, Html(content)).into_response()
}

pub async fn handle_wildcard(
    Path(path): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    info!(requested_path = %path, "Handling request");
    let extension = PathBuf::from(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("html")
        .to_lowercase();

    if extension.as_str() != "html" {
        debug!(path = %path, extension = %extension, "Serving static asset");
        return serve_static(&path, state.static_dir).await.into_response();
    }

    debug!(path = %path, "Serving HTML file");
    serve_html(&path, state.working_dir).await.into_response()
}

async fn serve_html(path: &str, dir_path: String) -> impl IntoResponse {
    let mut html_path = PathBuf::from(dir_path).join(path);

    if html_path.extension().is_none() {
        html_path.set_extension("html");
    }

    if !fs::try_exists(&html_path).await.unwrap_or(false) {
        warn!(path = %path, "HTML file not found");
        return (StatusCode::NOT_FOUND, Html(HTML_NOT_FOUND)).into_response();
    }

    debug!(file = ?html_path, "Reading HTML file");
    let file = fs::read_to_string(html_path).await;
    if file.is_err() {
        error!("Failed to read HTML file");
        return (StatusCode::INTERNAL_SERVER_ERROR, Html(HTML_INTERNAL_ERROR)).into_response();
    }

    let content = file.unwrap();
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        Html(content),
    )
        .into_response()
}

async fn serve_static(path: &str, dir_path: String) -> impl IntoResponse {
    let static_path = PathBuf::from(dir_path).join(path);

    if !fs::try_exists(&static_path).await.unwrap_or(false) {
        warn!(path = %path, "Static file not found");
        return (StatusCode::NOT_FOUND, Html(HTML_NOT_FOUND)).into_response();
    }

    match fs::read(&static_path).await {
        Ok(bytes) => {
            let mime_type = mime_guess::from_path(&static_path)
                .first_or_octet_stream()
                .to_string();
            debug!(path = %path, mime_type = %mime_type, bytes = bytes.len(), "Serving static file");
            (StatusCode::OK, [(header::CONTENT_TYPE, mime_type)], bytes).into_response()
        }
        Err(e) => {
            error!(path = %path, error = %e, "Failed to read static file");
            (StatusCode::INTERNAL_SERVER_ERROR, Html(HTML_INTERNAL_ERROR)).into_response()
        }
    }
}
