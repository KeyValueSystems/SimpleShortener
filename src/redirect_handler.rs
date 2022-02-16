use crate::structs::WebServerError;
use axum::extract::Path;

pub async fn redirect(
    Path(path): Path<String>,
) -> Result<(axum::http::StatusCode, axum::http::HeaderMap), WebServerError> {
    let destination_url_ref = crate::URLS
        .get()
        .ok_or(WebServerError::UrlsNotFound)?
        .get(&path)
        .ok_or(WebServerError::NotFound)?;
    let destination_url = destination_url_ref.as_str();
    tracing::trace!("Path: {}, Destination: {}", path, destination_url);
    let mut headers = axum::http::HeaderMap::new();
    let destination = match axum::http::HeaderValue::from_str(destination_url) {
        Ok(dest) => dest,
        Err(_) => return Err(WebServerError::InvalidRedirectUri),
    };
    headers.insert(axum::http::header::LOCATION, destination);
    Ok((axum::http::StatusCode::PERMANENT_REDIRECT, headers))
}

// Checks if a specific config var exists or serves the default root
pub async fn root() -> Result<
    (axum::http::StatusCode, axum::http::HeaderMap, &'static str),
    crate::structs::WebServerError,
> {
    tracing::trace!("Handling root request");
    let config = match crate::CONFIG.get() {
        None => return Err(crate::structs::WebServerError::ConfigNotFound),
        Some(config) => config,
    };
    let mut headers = axum::http::HeaderMap::new();
    match &config.root {
        None => {
            headers.insert(
                axum::http::header::CONTENT_TYPE,
                axum::http::HeaderValue::from_static("text/html"),
            );
            Ok((
                axum::http::StatusCode::OK,
                headers,
                include_str!("resources/root.html"),
            ))
        }
        Some(root) => {
            let destination = match axum::http::HeaderValue::from_str(root.as_str()) {
                Ok(dest) => dest,
                Err(_) => return Err(crate::structs::WebServerError::InvalidRedirectUri),
            };
            headers.insert(axum::http::header::LOCATION, destination);
            Ok((axum::http::StatusCode::PERMANENT_REDIRECT, headers, ""))
        }
    }
}
