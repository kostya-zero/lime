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

const ALLOWED_ASSETS_TYPES: [&str; 38] = [
    "jpg", "png", "jpeg", "gif", "svg", "webp", "ico", "bmp", "tiff", "avif", "css", "js", "mjs",
    "wasm", "ttf", "otf", "woff", "woff2", "eot", "mp3", "wav", "ogg", "m4a", "flac", "mp4",
    "webm", "ogm", "mov", "zip", "tar", "gz", "rar", "7z", "pdf", "txt", "csv", "xml", "json",
];

#[derive(Clone)]
pub struct AppState {
    working_dir: String,
    static_dir: String,
}

pub fn init_logging() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();
}

pub async fn start_server(config: &Config) -> Result<()> {
    println!(
        "\n {}{}",
        "\u{1F34B} Lime Web Server v".bright_green().bold(),
        env!("CARGO_PKG_VERSION").bright_green().bold()
    );
    let listener = TcpListener::bind(&format!("{}:{}", config.host, config.port))
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    let state = AppState {
        working_dir: config.working_dir.clone(),
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
        return (StatusCode::NOT_FOUND, Html(HTML_NOT_FOUND)).into_response();
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

    if ALLOWED_ASSETS_TYPES.contains(&extension.as_str()) {
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

    if html_path.exists() {
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
    } else {
        warn!(path = %path, "HTML file not found");
        (StatusCode::NOT_FOUND, Html(HTML_NOT_FOUND)).into_response()
    }
}

async fn serve_static(path: &str, dir_path: String) -> impl IntoResponse {
    let static_path = PathBuf::from(dir_path).join(path);

    if !static_path.exists() {
        warn!(path = %path, "Static file not found");
        return (StatusCode::NOT_FOUND, Html(HTML_NOT_FOUND)).into_response();
    }

    match fs::read(&static_path).await {
        Ok(bytes) => {
            let ext = static_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            let mime_type = match ext.as_str() {
                // Images
                "jpg" | "jpeg" => "image/jpeg",
                "png" => "image/png",
                "gif" => "image/gif",
                "svg" => "image/svg",
                "webp" => "image/webp",
                "ico" => "image/x-icon",
                "bmp" => "image/bmp",
                "tiff" => "image/tiff",
                "avif" => "image/avif",

                // Documents
                "pdf" => "application/pdf",
                "txt" => "text/plain",
                "csv" => "text/csv",
                "xml" => "application/xm",
                "json" => "application/json",

                // Web Assests
                "css" => "text/css",
                "js" => "application/javascript",
                "mjs" => "application/javascript",
                "wasm" => "application/wasm",

                // Fonts
                "ttf" => "font/ttf",
                "otf" => "font/otf",
                "woff" => "font/woff",
                "woff2" => "font/woff2",
                "eot" => "application/vnd.ms-fontobject",

                // Audio
                "mp3" => "audio/mpeg",
                "wav" => "audio/wav",
                "ogg" => "audio/ogg",
                "m4a" => "audio/mp4",
                "flac" => "audio/flac",

                // Video
                "mp4" => "video/mp4",
                "webm" => "video/webm",
                "ogm" => "video/ogg",
                "mov" => "video/quicktime",

                // If non of them matches
                _ => "application/octet-stream",
            };
            debug!(path = %path, mime_type = %mime_type, bytes = bytes.len(), "Serving static file");
            (StatusCode::OK, [(header::CONTENT_TYPE, mime_type)], bytes).into_response()
        }
        Err(e) => {
            error!(path = %path, error = %e, "Failed to read static file");
            (StatusCode::INTERNAL_SERVER_ERROR, Html(HTML_INTERNAL_ERROR)).into_response()
        }
    }
}
