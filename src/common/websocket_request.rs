use super::{AuthUser, ClientConfig};
use bytes::{BufMut, BytesMut};
use std::io::Error as IoError;
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::{self, SystemTime};
use thiserror::Error;
use tokio_tungstenite::tungstenite::{
    client::IntoClientRequest,
    handshake::client::Request,
    http::header::{self, HeaderName, HeaderValue},
    http::uri::InvalidUri,
    http::Uri,
    Result as WsResult,
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
    auth_user: AuthUser,
    ///服务端ip地址+端口
    pub server_addr: SocketAddr,
    ///是否跳过ssl证书验证
    pub insecure: bool,
    ///额外的http请求头
    pub extra_http_headers: Vec<(HeaderName, HeaderValue)>,
}

impl WebsocketRequest {
    fn gen_token_header(&self) -> (HeaderName, HeaderValue) {
        let time_now = SystemTime::now();
        let ts = time_now.duration_since(time::UNIX_EPOCH).unwrap().as_secs();
        let mut buf = BytesMut::with_capacity(self.auth_user.user.len() + 8 + 20);
        buf.put_slice(self.auth_user.user.as_bytes());
        buf.put_u64(ts);
        let token = self.auth_user.get_token(ts);
        buf.put_slice(&token);
        let encoded_buf = base64::encode(&buf);
        //Bearer token
        let token_value = format!("Bearer {encoded_buf}");
        let token_value: HeaderValue = token_value.parse().unwrap();
        (header::AUTHORIZATION, token_value)
    }
}

impl IntoClientRequest for &WebsocketRequest {
    fn into_client_request(self) -> WsResult<Request> {
        let mut request = (&self.server_uri).into_client_request()?;
        let headers = request.headers_mut();
        let token_header = self.gen_token_header();
        headers.insert(token_header.0, token_header.1);
        for (h_name, h_value) in &self.extra_http_headers {
            headers.insert(h_name, h_value.to_owned());
        }
        //dbg!(&request);
        Ok(request)
    }
}

impl TryFrom<&ClientConfig> for WebsocketRequest {
    type Error = ParseWebsocketRequestError;

    fn try_from(value: &ClientConfig) -> Result<Self, Self::Error> {
        let server_uri: Uri = value.server_url.parse()?;
        //获取host/ip
        let server_host = match &value.server_ip {
            Some(ip_value) if !ip_value.is_empty() => ip_value.as_str(),
            _ => match server_uri.host() {
                Some(s) => s,
                None => return Err(ParseWebsocketRequestError::UriNoHostErr),
            },
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
        //headers
        let extra_http_headers = match &value.extra_http_headers {
            Some(extra_headers) if !extra_headers.is_empty() => {
                let mut h_headers = Vec::with_capacity(extra_headers.len());
                for header_pair in extra_headers {
                    let h_name = header_pair[0].parse().unwrap();
                    let h_value = header_pair[1].parse().unwrap();
                    h_headers.push((h_name, h_value));
                }
                h_headers
            }
            _ => Vec::new(),
        };
        Ok(Self {
            server_uri,
            auth_user: value.auth_user.to_owned(),
            server_addr,
            insecure: matches!(value.insecure, Some(true)),
            extra_http_headers,
        })
    }
}
