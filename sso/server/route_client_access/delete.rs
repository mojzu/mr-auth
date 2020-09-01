use crate::server::internal::*;

#[api_v2_operation(summary = "Delete user access")]
pub async fn post(
    server: Data<Server>,
    req: HttpRequest,
    auth: BasicAuth,
    body: Json<RequestAccessDelete>,
) -> HttpResult<Json<()>> {
    server_request!(&server, &req, async {
        let body = server_validate!(&server, body);

        let client = server.client_required(auth).await?;

        let res = server.postgres.access_delete(&client, body).await;

        server.response_json(res)
    })
}
