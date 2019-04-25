use crate::api::{
    auth::{validate_name, validate_id}, authenticate, body_json_config, ApiData, ApiError, BodyFromValue,
};
use crate::models::AuthKey;
use actix_web::http::StatusCode;
use actix_web::middleware::identity::Identity;
use actix_web::{web, Error, HttpResponse, ResponseError};
use chrono::{DateTime, Utc};
use futures::{future, Future};
use validator::Validate;

/// Version 1 key routes scope.
pub fn v1_service() -> actix_web::Scope {
    web::scope("/key")
        .service(
            web::resource("")
                .route(web::get().to_async(v1_list))
                .route(web::post().data(body_json_config()).to_async(v1_create)),
        )
        .service(
            web::resource("/{key_id}")
                .route(web::get().to_async(v1_read))
                .route(web::patch().data(body_json_config()).to_async(v1_update))
                .route(web::delete().to_async(v1_delete)),
        )
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Key {
    pub created_at: DateTime<Utc>,
    pub id: i64,
    pub name: String,
    pub value: String,
    pub service_id: i64,
    pub user_id: Option<i64>,
}

impl From<AuthKey> for Key {
    fn from(key: AuthKey) -> Self {
        Key {
            created_at: key.created_at,
            id: key.key_id,
            name: key.key_name,
            value: key.key_value,
            service_id: key.service_id,
            user_id: key.user_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListQuery {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub order: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListResponse {
    pub data: Vec<Key>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateBody {
    #[validate(custom = "validate_name")]
    pub name: String,
    #[validate(custom = "validate_id")]
    pub user_id: i64,
}

impl BodyFromValue<CreateBody> for CreateBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateBody {
    pub name: Option<String>,
}

pub fn v1_list(
    data: web::Data<ApiData>,
    id: Identity,
    query: web::Query<ListQuery>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();
    let query = query.into_inner();

    list_inner(data, id, query).then(|r| match r {
        Ok(r) => future::ok(HttpResponse::Ok().json(r)),
        Err(e) => future::ok(e.error_response()),
    })
}

fn list_inner(
    data: web::Data<ApiData>,
    id: Option<String>,
    query: ListQuery,
) -> impl Future<Item = ListResponse, Error = ApiError> {
    web::block(move || {
        authenticate(&data, id).and_then(|service| {
            data.db
                .key_list(
                    query.offset,
                    query.limit,
                    query.order.as_ref().map(|x| &**x),
                    service.service_id,
                )
                .map_err(Into::into)
                .map(|keys| {
                    let data: Vec<Key> = keys.into_iter().map(Into::into).collect();
                    ListResponse { data }
                })
        })
    })
    .map_err(Into::into)
}

pub fn v1_create(
    data: web::Data<ApiData>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();

    CreateBody::from_value(body.into_inner())
        .and_then(move |body| create_inner(data, id, body))
        .then(|r| match r {
            Ok(r) => future::ok(HttpResponse::Ok().json(r)),
            Err(e) => future::ok(e.error_response()),
        })
}

fn create_inner(
    data: web::Data<ApiData>,
    id: Option<String>,
    body: CreateBody,
) -> impl Future<Item = Key, Error = ApiError> {
    web::block(move || {
        authenticate(&data, id).and_then(|service| {
            data.db
                .key_create(&body.name, service.service_id, Some(body.user_id))
                .map_err(Into::into)
                .map(Into::into)
        })
    })
    .map_err(Into::into)
}

pub fn v1_read(
    data: web::Data<ApiData>,
    id: Identity,
    path: web::Path<(i64,)>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();

    web::block(move || {
        authenticate(&data, id).and_then(|service| {
            data.db
                .key_read_by_id(path.0, service.service_id)
                .map_err(Into::into)
                .map(Into::into)
        })
    })
    .map_err(Into::into)
    .map(|key: Key| HttpResponse::build(StatusCode::OK).json(key))
}

pub fn v1_update(
    data: web::Data<ApiData>,
    id: Identity,
    path: web::Path<(i64,)>,
    body: web::Json<UpdateBody>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();
    let body = body.into_inner();

    web::block(move || {
        authenticate(&data, id).and_then(|service| {
            data.db
                .key_update_by_id(path.0, service.service_id, body.name.as_ref().map(|x| &**x))
                .map_err(Into::into)
                .map(Into::into)
        })
    })
    .map_err(Into::into)
    .map(|key: Key| HttpResponse::build(StatusCode::OK).json(key))
}

pub fn v1_delete(
    data: web::Data<ApiData>,
    id: Identity,
    path: web::Path<(i64,)>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();

    web::block(move || {
        authenticate(&data, id).and_then(|service| {
            data.db
                .key_delete_by_id(path.0, service.service_id)
                .map_err(Into::into)
        })
    })
    .map_err(Into::into)
    .map(|_| HttpResponse::new(StatusCode::OK))
}
