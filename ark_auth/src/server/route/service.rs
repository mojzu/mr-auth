use crate::{
    server::{
        route::{request_audit_meta, route_response_empty, route_response_json},
        Data,
    },
    server_api::{
        path, ServiceCreateBody, ServiceListQuery, ServiceListResponse, ServiceReadResponse,
        ServiceUpdateBody,
    },
    AuditMeta, Key, ServerError, ServerResult, ServerValidateFromValue, Service, ServiceQuery,
};
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use futures::Future;
use serde_json::Value;
use uuid::Uuid;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope(path::SERVICE)
        .service(
            web::resource(path::NONE)
                .route(web::get().to_async(list_handler))
                .route(web::post().to_async(create_handler)),
        )
        .service(
            web::resource(path::ID)
                .route(web::get().to_async(read_handler))
                .route(web::patch().to_async(update_handler))
                .route(web::delete().to_async(delete_handler)),
        )
}

fn list_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    query: web::Query<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let query = ServiceListQuery::from_value(query.into_inner());

    audit_meta
        .join(query)
        .and_then(|(audit_meta, query)| {
            web::block(move || list_inner(data.get_ref(), audit_meta, id, query.into()))
                .map_err(Into::into)
        })
        .then(route_response_json)
}

fn list_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    query: ServiceQuery,
) -> ServerResult<ServiceListResponse> {
    Key::authenticate_root(data.driver(), audit_meta, id)
        .and_then(|mut audit| {
            let service_ids = Service::list(data.driver(), &mut audit, &query)?;
            Ok(ServiceListResponse {
                meta: query,
                data: service_ids,
            })
        })
        .map_err(Into::into)
}

fn create_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = ServiceCreateBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            web::block(move || create_inner(data.get_ref(), audit_meta, id, &body))
                .map_err(Into::into)
        })
        .then(route_response_json)
}

fn create_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    body: &ServiceCreateBody,
) -> ServerResult<ServiceReadResponse> {
    Key::authenticate_root(data.driver(), audit_meta, id)
        .and_then(|mut audit| {
            Service::create(
                data.driver(),
                &mut audit,
                body.is_enabled,
                &body.name,
                &body.url,
            )
        })
        .map_err(Into::into)
        .map(|service| ServiceReadResponse { data: service })
}

fn read_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    path: web::Path<(Uuid,)>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);

    audit_meta
        .and_then(|audit_meta| {
            web::block(move || read_inner(data.get_ref(), audit_meta, id, path.0))
                .map_err(Into::into)
        })
        .then(route_response_json)
}

fn read_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    service_id: Uuid,
) -> ServerResult<ServiceReadResponse> {
    Key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            Service::read_by_id(data.driver(), service.as_ref(), &mut audit, service_id)
        })
        .map_err(Into::into)
        .and_then(|service| service.ok_or_else(|| ServerError::NotFound))
        .map(|service| ServiceReadResponse { data: service })
}

fn update_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    path: web::Path<(Uuid,)>,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = ServiceUpdateBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            web::block(move || update_inner(data.get_ref(), audit_meta, id, path.0, &body))
                .map_err(Into::into)
        })
        .then(route_response_json)
}

fn update_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    service_id: Uuid,
    body: &ServiceUpdateBody,
) -> ServerResult<ServiceReadResponse> {
    Key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            Service::update_by_id(
                data.driver(),
                service.as_ref(),
                &mut audit,
                service_id,
                body.is_enabled,
                body.name.as_ref().map(|x| &**x),
            )
        })
        .map_err(Into::into)
        .map(|service| ServiceReadResponse { data: service })
}

fn delete_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    path: web::Path<(Uuid,)>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);

    audit_meta
        .and_then(|audit_meta| {
            web::block(move || delete_inner(data.get_ref(), audit_meta, id, path.0))
                .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn delete_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    service_id: Uuid,
) -> ServerResult<usize> {
    Key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            Service::delete_by_id(data.driver(), service.as_ref(), &mut audit, service_id)
        })
        .map_err(Into::into)
}
