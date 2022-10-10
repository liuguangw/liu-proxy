use std::{num::ParseIntError, str::Utf8Error};

use http::{self, uri::InvalidUri, Uri};
use httparse::Header;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseRequestError {
    #[error("parse length value as utf-8 str failed, {0}")]
    Utf8Err(#[from] Utf8Error),
    #[error("parse length value as number failed, {0}")]
    ParseIntErr(#[from] ParseIntError),
    #[error("parse request uri failed, {0}")]
    ParseUriErr(#[from] InvalidUri),
    #[error("host value not found in uri")]
    UriNoHost,
    #[error("unsupport scheme {0}")]
    UnSupportScheme(String),
    #[error("scheme value not found in uri")]
    NoScheme,
}

///根据header计算request body的完整大小
pub fn parse_request_body_size(headers: &[Header<'_>]) -> Result<usize, ParseRequestError> {
    let mut body_size = 0;
    for header_info in headers {
        if header_info.name == http::header::CONTENT_LENGTH {
            let length_str = std::str::from_utf8(header_info.value)?;
            body_size = length_str.parse()?;
            break;
        }
    }
    Ok(body_size)
}

///不是connect请求时,获取目标地址端口
pub fn parse_conn_dest(url: &str) -> Result<String, ParseRequestError> {
    let request_uri: Uri = url.parse()?;
    let host = match request_uri.host() {
        Some(s) => s,
        None => return Err(ParseRequestError::UriNoHost),
    };
    let port = match request_uri.port_u16() {
        Some(s) => s,
        None => match request_uri.scheme_str() {
            Some("http") => 80,
            Some(o) => return Err(ParseRequestError::UnSupportScheme(o.to_string())),
            None => return Err(ParseRequestError::NoScheme),
        },
    };
    let conn_dest = format!("{host}:{port}");
    Ok(conn_dest)
}
