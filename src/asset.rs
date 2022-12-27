use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/public/"]
#[prefix = "/"]
struct Assets;

pub async fn handler(uri: Uri) -> Result<impl IntoResponse, StatusCode> {
    let path = uri.path();
    let path = if path.ends_with('/') {
        format!("{}index.html", path)
    } else {
        path.to_owned()
    };

    let mime_type = match path.rsplit('.').next() {
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("png") => "image/png",
        _ => "text/plain",
    };

    let asset = Assets::get(&path).ok_or(StatusCode::NOT_FOUND)?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type)
        .body(Body::from(asset.data))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
}
