use super::proxy_error::ProxyError;
use crate::common::ProxyRequestResult;
use crate::common::ProxyResponseResult;
use crate::services::poll_message;
use crate::services::read_raw_data;
use futures_util::SinkExt;
use std::io::{Error as IoError, ErrorKind};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::WebSocketStream;

pub async fn proxy_tcp(
    client_stream: &mut WebSocketStream<TcpStream>,
    mut remote_stream: TcpStream,
) -> Result<(), ProxyError> {
    loop {
        tokio::select! {
            //读取客户端请求
            request_result = poll_message::poll_binary_message(client_stream) => {
                let loop_option = proxy_send_request(request_result,&mut remote_stream,client_stream).await?;
                if loop_option.is_some(){
                    return Ok(());
                }
            },
            //读取远端响应
            response_result = read_raw_data::read_raw(&mut remote_stream) => {
                let loop_option = proxy_send_response(response_result,client_stream).await?;
                if loop_option.is_some(){
                    return Ok(());
                }
            }
        };
    }
}

///将客户端请求转发给远端
///
/// - 如果 `Err` ,表示读取客户端请求出错,或者把请求结果转发给客户端出错
/// - 如果 `Option<()>` 为 `()` ,表示请求远端出错(也有可能是远端主动关闭了tcp连接)
/// - 如果 `Option<()>` 为 `None` ,一切正常
async fn proxy_send_request(
    read_request_result: Result<Vec<u8>, WsError>,
    remote_stream: &mut TcpStream,
    client_stream: &mut WebSocketStream<TcpStream>,
) -> Result<Option<()>, ProxyError> {
    //读取客户端请求
    let request_data = read_request_result
    .map_err(|e|ProxyError::ws_err("read client request", e))?;
    let (request_result, loop_option) = match remote_stream.write(&request_data).await {
        Ok(_) => (ProxyRequestResult::Ok, None),
        Err(e) => {
            let r = if e.kind() == ErrorKind::UnexpectedEof {
                ProxyRequestResult::Closed
            } else {
                ProxyRequestResult::Err(e)
            };
            (r, Some(()))
        }
    };
    let request_ret_msg = request_result.message();
    //把write远端的结果发给客户端
    if let Err(e) = client_stream.send(request_ret_msg).await {
        return Err(ProxyError::ws_err("write request result", e));
    }
    Ok(loop_option)
}

///将响应转发给客户端
///
/// - 如果 `Err` ,表示把 `response` 转发给客户端出错
/// - 如果 `Option<()>` 为 `()` ,表示读取远端response出错(也有可能是远端主动关闭了tcp连接)
/// - 如果 `Option<()>` 为 `None` ,一切正常
async fn proxy_send_response(
    read_response_result: Result<Vec<u8>, IoError>,
    client_stream: &mut WebSocketStream<TcpStream>,
) -> Result<Option<()>, ProxyError> {
    let (response_result, loop_option) = match read_response_result {
        Ok(data) => (ProxyResponseResult::Ok(data), None),
        Err(e) => {
            let r = if  e.kind() == ErrorKind::UnexpectedEof {
                ProxyResponseResult::Closed
            } else {
                ProxyResponseResult::Err(e)
            };
            (r, Some(()))
        }
    };
    let response_msg = response_result.message();
    //把write远端的结果发给客户端
    if let Err(e) = client_stream.send(response_msg).await {
        return Err(ProxyError::ws_err("write response to client", e));
    }
    Ok(loop_option)
}
