use crate::structs::{Add, Authorization, Delete, Edit, List, WebServerError};
use axum::http::StatusCode;
use axum::Json;

pub async fn list(_: crate::structs::Authorization) -> Result<Json<List>, WebServerError> {
    Ok(Json(List {
        links: crate::URLS
            .get()
            .ok_or(WebServerError::UrlsNotFound)?
            .clone(),
    }))
}

pub async fn edit(
    _: Authorization,
    Json(Edit { link, destination }): Json<Edit>,
) -> Result<&'static str, WebServerError> {
    let links = crate::URLS.get().ok_or(WebServerError::UrlsNotFound)?;
    let _: () = links
        .contains_key(&link)
        .then(|| ())
        .ok_or(WebServerError::NotFoundJson)?;

    let _: () = crate::DISALLOWED_SHORTENINGS
        .get()
        .ok_or(WebServerError::DisallowedNotFound)?
        .contains(&link)
        .then(|| ())
        .ok_or(WebServerError::UrlConflict)?;

    let db = crate::DB.get().ok_or(WebServerError::DbNotFound)?;
    assert_ne!(
        sqlx::query!(
            "UPDATE links SET destination = $1 WHERE link = $2",
            destination,
            link
        )
        .execute(db)
        .await?
        .rows_affected(),
        1,
        "already checked there would be at least one row in the database but that row does not exist?"
    );
    links.insert(link, destination);

    Ok(r#"{"message":"Link edited!"}\n"#)
}

pub async fn delete(
    _: Authorization,
    Json(Delete { link }): Json<Delete>,
) -> Result<&'static str, WebServerError> {
    let links = crate::URLS.get().ok_or(WebServerError::UrlsNotFound)?;
    let _: () = links
        .contains_key(&link)
        .then(|| ())
        .ok_or(WebServerError::NotFoundJson)?;

    let db = crate::DB.get().ok_or(WebServerError::DbNotFound)?;
    assert_ne!(
        sqlx::query!("DELETE FROM links WHERE link = $1", link)
            .execute(db)
            .await?
            .rows_affected(),
        1,
        "already checked there would be at least one row in the database but that row does not exist?"
    );
    links.remove(&link);

    Ok(r#"{"message":"Link removed!"}"#)
}

pub async fn add(
    _: Authorization,
    Json(Add { link, destination }): Json<Add>,
) -> Result<(StatusCode, &'static str), WebServerError> {
    let links = crate::URLS.get().ok_or(WebServerError::UrlsNotFound)?;
    let _: () = !(links.contains_key(&link))
        .then(|| ())
        .ok_or(WebServerError::UrlConflict)?;

    let _: () = crate::DISALLOWED_SHORTENINGS
        .get()
        .ok_or(WebServerError::DisallowedNotFound)?
        .contains(&link)
        .then(|| ())
        .ok_or(WebServerError::UrlConflict)?;

    let db = crate::DB.get().ok_or(WebServerError::DbNotFound)?;

    sqlx::query!("INSERT INTO links VALUES ($1, $2)", link, destination)
        .execute(db)
        .await?;

    links.insert(link, destination);

    Ok((StatusCode::CREATED, r#"{"message":"Link added!"}"#))
}
