use anyhow::{Result, anyhow};
use axum::{
    Router,
    extract::Path,
    http::{StatusCode, header},
    response::{Html, IntoResponse},
    routing::get,
};
use std::path::PathBuf;
use tokio::{fs, net::TcpListener};

const HTML_NOT_FOUND: &str = include_str!("../static/not-found.html");
const HTML_INTERNAL_ERROR: &str = include_str!("../static/internal-error.html");

const ALLOWED_ASSETS_TYPES: [&str; 38] = [
    "jpg", "png", "jpeg", "gif", "svg", "webp", "ico", "bmp", "tiff", "avif", "css", "js", "mjs",
    "wasm", "ttf", "otf", "woff", "woff2", "eot", "mp3", "wav", "ogg", "m4a", "flac", "mp4",
    "webm", "ogm", "mov", "zip", "tar", "gz", "rar", "7z", "pdf", "txt", "csv", "xml", "json",
];

pub async fn start_server(port: u16) -> Result<()> {
    let listener = TcpListener::bind(&format!("0.0.0.0:{port}"))
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    let router = Router::new()
        .route("/", get(handle_index))
        .route("/{*path}", get(handle_wildcard));

    axum::serve(listener, router)
        .await
        .map_err(|e| anyhow!(e.to_string()))?;

    Ok(())
}

pub async fn handle_index() -> impl IntoResponse {
    let path = PathBuf::from("index.html");
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

pub async fn handle_wildcard(Path(path): Path<String>) -> impl IntoResponse {
    println!("Requested path: {path}");
    let extension = PathBuf::from(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("html")
        .to_lowercase();

    if ALLOWED_ASSETS_TYPES.contains(&extension.as_str()) {
        println!("Serving static: {path}");
        return serve_static(&path).await.into_response();
    }

    println!("Serving HTML file: {path}");
    serve_html(&path).await.into_response()
}

async fn serve_html(path: &str) -> impl IntoResponse {
    let mut html_path = PathBuf::from(path);

    if html_path.extension().is_none() {
        html_path.set_extension("html");
    }

    if html_path.exists() {
        let file = fs::read_to_string(html_path).await;
        if file.is_err() {
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
        (StatusCode::NOT_FOUND, Html(HTML_NOT_FOUND)).into_response()
    }
}

async fn serve_static(path: &str) -> impl IntoResponse {
    let static_path = PathBuf::from("public/").join(path);

    if !static_path.exists() {
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
            (StatusCode::OK, [(header::CONTENT_TYPE, mime_type)], bytes).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Html(HTML_INTERNAL_ERROR)).into_response(),
    }
}
