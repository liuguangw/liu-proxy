use super::{AuthUser, ClientConfig};
use bytes::{BufMut, BytesMut};
use rustls::{ClientConfig as SslClientConfig, OwnedTrustAnchor, RootCertStore};
use std::fs;
use std::io::Error as IoError;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
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
use tokio_tungstenite::Connector;
use webpki::{Error as WebpkiError, TrustAnchor};

///根据客户端配置,构造握手请求数据 [`WebsocketRequest`] 出现的错误
#[derive(Error, Debug)]
pub enum ParseWebsocketRequestError {
    ///url格式错误
    #[error("parse websocket uri failed: {0}")]
    UriErr(#[from] InvalidUri),
    ///url中缺少host部分
    #[error("parse websocket uri failed: host field not found")]
    UriNoHostErr,
    ///协议错误(只能是ws/wss)
    #[error("invalid scheme")]
    InvalidScheme,
    ///解析服务端地址出错
    #[error("parse address failed: {0}")]
    ParseAddrErr(IoError),
    ///无法解析出有效的地址
    #[error("resolve address failed")]
    ResolveErr,
    ///读取ca文件出错
    #[error("read ca file failed: {0}")]
    ReadCaFileErr(IoError),
    ///解析ca出错
    #[error("parse trust anchor failed: {0}")]
    ParseTrustAnchorErr(#[from] WebpkiError),
}

///客户端发起的握手请求
pub struct WebsocketRequest {
    server_uri: Uri,
    auth_user: AuthUser,
    ///服务端ip地址+端口
    pub server_addr: SocketAddr,
    pub ssl_connector: Option<Connector>,
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

    ///加载ca列表
    fn load_ca_from_file(
        ca_path: &str,
    ) -> Result<Vec<OwnedTrustAnchor>, ParseWebsocketRequestError> {
        let file_content = fs::read(ca_path).map_err(ParseWebsocketRequestError::ReadCaFileErr)?;
        let mut data = file_content.as_slice();
        let items = rustls_pemfile::read_all(&mut data)
            .map_err(ParseWebsocketRequestError::ReadCaFileErr)?;
        let mut ca_list = Vec::new();
        for item in items {
            match item {
                rustls_pemfile::Item::X509Certificate(der_data) => {
                    let trust_anchor = TrustAnchor::try_from_cert_der(&der_data)?;
                    let owned_trust_anchor = OwnedTrustAnchor::from_subject_spki_name_constraints(
                        trust_anchor.subject,
                        trust_anchor.spki,
                        trust_anchor.name_constraints,
                    );
                    ca_list.push(owned_trust_anchor);
                }
                _ => continue,
            }
        }
        Ok(ca_list)
    }

    ///根据ca文件,构造ssl连接配置
    fn build_ssl_config(ca_path: &str) -> Result<SslClientConfig, ParseWebsocketRequestError> {
        let mut root_store = RootCertStore::empty();
        let trust_anchors = Self::load_ca_from_file(ca_path)?;
        root_store.add_server_trust_anchors(trust_anchors.into_iter());
        let config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        Ok(config)
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
        let ssl_connector = match &value.ssl_ca_path {
            Some(ca_path) => {
                let ssl_config = Self::build_ssl_config(ca_path)?;
                let connector = Connector::Rustls(Arc::new(ssl_config));
                Some(connector)
            }
            None => None,
        };
        Ok(Self {
            server_uri,
            auth_user: value.auth_user.to_owned(),
            server_addr,
            ssl_connector,
            extra_http_headers,
        })
    }
}
