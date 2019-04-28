pub mod github;
pub mod microsoft;

use crate::api::{ApiData, ApiError};
use crate::db::TokenData;
use crate::models::AuthService;
use actix_web::http::{header, StatusCode};
use actix_web::{web, HttpResponse};
use url::Url;

// TODO(feature): Other OAuth2 providers support.

/// Version 1 authentication OAuth2 routes scope.
pub fn v1_service() -> actix_web::Scope {
    web::scope("/oauth2")
        .service(
            web::resource("/github")
                .route(web::post().to_async(github::v1))
                .route(web::get().to_async(github::v1_callback)),
        )
        .service(
            web::resource("/microsoft")
                .route(web::post().to_async(microsoft::v1))
                .route(web::get().to_async(microsoft::v1_callback)),
        )
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UrlResponse {
    pub url: String,
}

pub fn oauth2_login(
    data: &web::Data<ApiData>,
    email: &str,
    service_id: i64,
) -> Result<(TokenData, AuthService), ApiError> {
    let token = data
        .db
        .oauth2_login(email, service_id)
        .map_err(ApiError::Db)?;
    let service = data
        .db
        .service_read_by_id(service_id, service_id)
        .map_err(ApiError::Db)?;
    Ok((token, service))
}

pub fn oauth2_redirect(token: TokenData, service: AuthService) -> HttpResponse {
    let mut url = Url::parse(&service.service_url).unwrap();
    let token_query = format!("token={}", token.token);
    url.set_query(Some(&token_query));

    HttpResponse::build(StatusCode::FOUND)
        .header(header::LOCATION, url.as_str())
        .finish()
}
