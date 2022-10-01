use super::proxy_error::ProxyError;
use crate::services::{poll_message, read_raw_data};
use futures_util::{SinkExt, StreamExt};
use std::io::{Error as IoError, ErrorKind};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::Result as WsResult;

pub async fn proxy_tcp<T>(ws_stream: &mut T, tcp_stream: &mut TcpStream) -> Result<(), ProxyError>
where
    T: StreamExt<Item = WsResult<Message>> + SinkExt<Message, Error = WsError> + Unpin,
{
    loop {
        tokio::select! {
            //读取客户端请求
            request_result =  read_raw_data::read_raw(tcp_stream) => {
                proxy_send_request(request_result,ws_stream).await?;
            },
            //读取服务端响应
            response_result =poll_message::poll_binary_message(ws_stream) => {
                proxy_send_response(response_result,tcp_stream).await?;
            }
        }
    }
}

///将客户端请求转发给远端
///
/// - 如果 `IoError` ,表示读取请求出错
/// - 如果 `WsError` ,表示把请求结果转发给服务端出错
/// - 如果 `Ok` ,一切正常
async fn proxy_send_request<T>(
    read_request_result: Result<Vec<u8>, IoError>,
    ws_stream: &mut T,
) -> Result<(), ProxyError>
where
    T: SinkExt<Message, Error = WsError> + Unpin,
{
    //读取客户端请求
    //todo 客户端主动断开的处理
    let request_data = read_request_result.map_err(|e| ProxyError::io_err("read request", e))?;
    let request_msg = Message::binary(request_data);
    //把请求发给服务端
    if let Err(e) = ws_stream.send(request_msg).await {
        return Err(ProxyError::ws_err("send request to server", e));
    }
    Ok(())
}

///将响应转发给客户端
///
/// - 如果 `Err` ,表示把 `response` 转发给客户端出错
/// - 如果 `Option<()>` 为 `()` ,表示读取远端response出错(也有可能是远端主动关闭了tcp连接)
/// - 如果 `Option<()>` 为 `None` ,一切正常
async fn proxy_send_response(
    read_response_result: Result<Vec<u8>, WsError>,
    tcp_stream: &mut TcpStream,
) -> Result<(), ProxyError> {
    let response_data = read_response_result.map_err(|e| ProxyError::ws_err("read server response", e))?;
    let mut iter = response_data.iter();
    let msg_type = match iter.next() {
        Some(s) => *s,
        None => {
            let err = IoError::new(ErrorKind::Other, "none msg type");
            return Err(ProxyError::io_err("parse msg type", err));
        }
    };
    let msg_code = match iter.next() {
        Some(s) => *s,
        None => {
            let err = IoError::new(ErrorKind::Other, "none msg code");
            return Err(ProxyError::io_err("parse msg code", err));
        }
    };
    if msg_code != 0 {
        let err = if msg_type == 0 {
            IoError::new(ErrorKind::Other, "server request error")
        } else if msg_type == 1 {
            IoError::new(ErrorKind::Other, "server response error")
        } else {
            IoError::new(ErrorKind::Other, "unkown error")
        };
        return Err(ProxyError::io_err("ret code state", err));
    }
    if msg_type == 1 {
        let data: Vec<u8> = iter.copied().collect();
        if let Err(err) = tcp_stream.write(&data).await {
            return Err(ProxyError::io_err("write response", err));
        }
    }
    Ok(())
}
