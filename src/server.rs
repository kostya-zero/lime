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

const ALLOWED_IMAGE_TYPES: [&str; 5] = ["jpg", "png", "jpeg", "gif", "webp"];

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
    let mut path = PathBuf::from(path);
    let extension = if let Some(ext) = path.extension() {
        ext.to_str().unwrap().to_string()
    } else {
        String::from("html")
    };
    path.set_extension(&extension);

    if extension != "html" && ALLOWED_IMAGE_TYPES.contains(&extension.as_str()) {
        let static_path = PathBuf::from("public/").join(&path);
        let file = fs::read(&static_path).await;
        if file.is_err() {
            return (StatusCode::INTERNAL_SERVER_ERROR, Html(HTML_INTERNAL_ERROR)).into_response();
        }

        let bytes = file.unwrap();
        return (
            [(header::CONTENT_TYPE, format!("image/{extension}"))],
            bytes,
        )
            .into_response();
    }

    if !path.exists() {
        return (StatusCode::NOT_FOUND, Html(HTML_NOT_FOUND)).into_response();
    }

    let file = fs::read_to_string(path).await;
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
}
