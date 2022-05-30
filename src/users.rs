use axum::http::HeaderMap;
use axum::{Extension, Json};
use serde_json::Value;
use sha2::Digest;
use sqlx::PgPool;

use crate::error::WebServerError;
use crate::structs::AddUser;
use crate::{TokenMap, UrlMap};
use std::sync::atomic::Ordering;
use std::sync::Arc;
pub struct Authorization;

#[async_trait::async_trait]
impl<T: Send> axum::extract::FromRequest<T> for Authorization {
    type Rejection = WebServerError;
    async fn from_request(
        req: &mut axum::extract::RequestParts<T>,
    ) -> Result<Self, Self::Rejection> {
        let headers = req.headers();
        let provided_token = String::from_utf8(
            headers
                .get("Authorization")
                .ok_or(WebServerError::BadRequest)?
                .as_bytes()
                .into(),
        )
        .map_err(|_| WebServerError::BadRequest)?;
        let tokens = req
            .extensions()
            .get::<Arc<TokenMap>>()
            .ok_or(WebServerError::MissingExtensions)?;
        if tokens.get(&provided_token).is_some() {
            Ok(Self)
        } else {
            Err(WebServerError::IncorrectAuth)
        }
    }
}

#[allow(clippy::unused_async)]
pub async fn token(
    headers: HeaderMap,
    Extension(db): Extension<PgPool>,
    Extension(tokens): Extension<UrlMap>,
) -> Result<Json<Value>, WebServerError> {
    let username = String::from_utf8(
        headers
            .get("username")
            .ok_or(WebServerError::BadRequest)?
            .as_bytes()
            .into(),
    )
    .map_err(|_| WebServerError::BadRequest)?;
    let password = String::from_utf8(
        headers
            .get("password")
            .ok_or(WebServerError::BadRequest)?
            .as_bytes()
            .into(),
    )
    .map_err(|_| WebServerError::BadRequest)?;
    let correct_with_salt = match query!(
        "SELECT password FROM accounts WHERE username = $1",
        &username
    )
    .fetch_one(&db)
    .await
    {
        Ok(pw) => pw,
        Err(sqlx::Error::RowNotFound) => {
            return Ok(Json(json!({ "error": "Username or password incorrect!" })))
        }
        Err(e) => return Err(WebServerError::Db(e)),
    };
    let correct_split = correct_with_salt.password.split('|').collect::<Vec<&str>>();
    let correct_hash = correct_split.get(1).ok_or(WebServerError::NoSalt)?;
    let salt = correct_split.get(0).ok_or(WebServerError::NoSalt)?;
    let provided_password_hash = sha2::Sha512::digest(&format!("{}|{}", salt, password))
        .into_iter()
        .map(|x| format!("{:02x}", x))
        .collect::<String>();
    if *correct_hash == provided_password_hash {
        return Ok(Json(json!({ "error": "Username or password incorrect!" })));
    };
    let token = crate::randstr();
    tokens.insert(username, token.clone());
    Ok(Json(json!({ "token": token })))
}

pub async fn setup(
    _: Authorization,
    Json(AddUser { username, password }): Json<AddUser>,
    Extension(db): Extension<&PgPool>,
    axum::Extension(is_init): axum::Extension<crate::IsInit>,
) -> Result<Json<Value>, WebServerError> {
    let salt = crate::randstr();
    let password_hash = sha2::Sha512::digest(&format!("{}|{}", &salt, password))
        .into_iter()
        .map(|x| format!("{:02x}", x))
        .collect::<String>();

    debug!("Creating new user with username {} and password hash `{}|{}`", &username, &salt, &password_hash);
    query!(
        "INSERT INTO accounts VALUES ($1, $2)",
        username,
        &format!("{}|{}", &salt, password_hash)
    )
    .execute(db)
    .await?;
    is_init.store(true, Ordering::Relaxed);
    Ok(Json(json!({"message":"Account added!"})))
}