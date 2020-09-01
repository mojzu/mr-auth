use crate::server::internal::*;

#[api_v2_operation(summary = "Delete user API key")]
pub async fn post(
    server: Data<Server>,
    req: HttpRequest,
    auth: BasicAuth,
    body: Json<RequestApiKeyDelete>,
) -> HttpResult<Json<()>> {
    server_request!(&server, &req, async {
        let body = server_validate!(&server, body);

        let client = server.client_required(auth).await?;

        let res = server.postgres.api_key_delete(&client, body).await;

        server.response_json(res)
    })
}
