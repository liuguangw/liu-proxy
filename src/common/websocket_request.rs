use super::ClientConfig;
use std::io::Error as IoError;
use std::net::{SocketAddr, ToSocketAddrs};
use thiserror::Error;
use tokio_tungstenite::tungstenite::{
    client::IntoClientRequest, handshake::client::Request, http::header, http::uri::InvalidUri,
    http::Uri, Result as WsResult,
};

#[derive(Error, Debug)]
pub enum ParseWebsocketRequestError {
    #[error("parse websocket uri failed: {0}")]
    UriErr(#[from] InvalidUri),
    #[error("parse websocket uri failed: host field not found")]
    UriNoHostErr,
    #[error("invalid scheme")]
    InvalidScheme,
    #[error("parse address failed: {0}")]
    ParseAddrErr(IoError),
    #[error("resolve address failed")]
    ResolveErr,
}

///客户端发起的握手请求
#[derive(Clone)]
pub struct WebsocketRequest {
    server_uri: Uri,
    ///服务端ip地址+端口
    pub server_addr: SocketAddr,
    ///是否跳过ssl证书验证
    pub insecure: bool,
    auth_token: String,
}

impl IntoClientRequest for &WebsocketRequest {
    fn into_client_request(self) -> WsResult<Request> {
        let mut request = (&self.server_uri).into_client_request()?;
        //Bearer token
        let token_value = format!("Bearer {}", self.auth_token);
        request
            .headers_mut()
            .insert(header::AUTHORIZATION, token_value.parse().unwrap());
        Ok(request)
    }
}

impl TryFrom<&ClientConfig> for WebsocketRequest {
    type Error = ParseWebsocketRequestError;

    fn try_from(value: &ClientConfig) -> Result<Self, Self::Error> {
        let server_uri: Uri = value.server_url.parse()?;
        //获取host/ip
        let server_host = if !value.server_ip.is_empty() {
            &value.server_ip
        } else {
            match server_uri.host() {
                Some(s) => s,
                None => return Err(ParseWebsocketRequestError::UriNoHostErr),
            }
        };
        //解析端口
        let server_port = match server_uri.port_u16() {
            Some(s) => s,
            None => match server_uri.scheme_str() {
                Some("ws") => 80,
                Some("wss") => 443,
                _ => return Err(ParseWebsocketRequestError::InvalidScheme),
            },
        };
        //解析ip地址
        let addrs_iter = (server_host, server_port)
            .to_socket_addrs()
            .map_err(ParseWebsocketRequestError::ParseAddrErr)?;
        //可能会有多个地址
        let server_addr = {
            let mut tmp_addr = None;
            for sock_addr in addrs_iter {
                //ipv4优先
                if sock_addr.is_ipv4() {
                    tmp_addr = Some(sock_addr);
                    break;
                }
                if tmp_addr.is_none() {
                    tmp_addr = Some(sock_addr);
                }
            }
            match tmp_addr {
                Some(s) => s,
                None => return Err(ParseWebsocketRequestError::ResolveErr),
            }
        };
        Ok(Self {
            server_uri,
            server_addr,
            insecure: value.insecure,
            auth_token: value.auth_token.to_owned(),
        })
    }
}
