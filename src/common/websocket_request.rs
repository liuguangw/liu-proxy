use tokio_tungstenite::tungstenite::{
    client::IntoClientRequest, handshake::client::Request, http::header, Result as WsResult,
};

///客户端发起的握手请求
pub struct WebsocketRequest<'a, 'b> {
    server_url: &'a str,
    auth_token: &'b str,
}

impl<'a, 'b> WebsocketRequest<'a, 'b> {
    ///以服务端url和token初始化
    pub fn new(server_url: &'a str, auth_token: &'b str) -> Self {
        Self {
            server_url,
            auth_token,
        }
    }
}

impl<'a, 'b> IntoClientRequest for WebsocketRequest<'a, 'b> {
    fn into_client_request(self) -> WsResult<Request> {
        let mut request = self.server_url.into_client_request()?;
        //Bearer token
        let token_value = format!("Bearer {}", self.auth_token);
        request
            .headers_mut()
            .insert(header::AUTHORIZATION, token_value.parse().unwrap());
        Ok(request)
    }
}
