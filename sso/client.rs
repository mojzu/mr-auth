use crate::internal::*;

/// Client
#[derive(Debug, Clone)]
pub struct Client {
    pub server_authorize_uri: Url,
    pub server_token_uri: Url,
    pub server_introspect_uri: Url,
    pub client_id: Uuid,
    pub client_secret: String,
    pub redirect_uri: Url,
    pub client_name: String,
    pub client_uri: Url,
    pub enable: bool,
    pub scope: oauth2::Scope,
    pub user_scope: oauth2::Scope,
    pub register_enable: bool,
    pub register_scope: oauth2::Scope,
    pub ttl: ConfigOauth2ClientTtl,
    pub templates: ConfigOauth2ClientTemplates,
}

impl Client {
    pub fn client_name(&self) -> String {
        self.client_name.clone()
    }

    pub fn client_uri(&self) -> Url {
        self.client_uri.clone()
    }
}

impl oauth2::ClientIf for Client {
    fn server_authorize_uri(&self) -> Url {
        self.server_authorize_uri.clone()
    }
    fn server_token_uri(&self) -> Url {
        self.server_token_uri.clone()
    }
    fn client_id(&self) -> String {
        self.client_id.to_string()
    }
    fn client_secret(&self) -> String {
        self.client_secret.clone()
    }
    fn redirect_uri(&self) -> Url {
        self.redirect_uri.clone()
    }
}

impl oauth2::ResourceServerIf for Client {
    fn server_introspect_uri(&self) -> Url {
        self.server_introspect_uri.clone()
    }
    fn client_id(&self) -> String {
        self.client_id.to_string()
    }
    fn client_secret(&self) -> String {
        self.client_secret.clone()
    }
}
