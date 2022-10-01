use super::io::ProxyRequestResult;
use super::io::ProxyResponseResult;
use super::proxy_error::ProxyError;
use crate::services::poll_message;
use crate::services::read_raw_data;
use futures_util::SinkExt;
use std::io::{Error as IoError, ErrorKind};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

pub async fn proxy_tcp(
    client_stream: &mut WebSocketStream<TcpStream>,
    mut remote_stream: TcpStream,
) -> Result<(), ProxyError> {
    loop {
        tokio::select! {
            //读取客户端请求
            request_result = poll_message::poll_binary_message(client_stream) => {
                let send_request_result = proxy_send_request(request_result,&mut remote_stream,client_stream).await?;
                if !send_request_result.is_ok(){
                    return Ok(());
                }
            },
            //读取远端响应
            response_result = read_raw_data::read_raw(&mut remote_stream) => {
                let send_response_result = proxy_send_response(response_result,client_stream).await?;
                if !send_response_result.is_ok(){
                    return Ok(());
                }
            }
        };
    }
}

///将客户端请求转发给远端
async fn proxy_send_request(
    read_request_result: Result<Option<Vec<u8>>, WsError>,
    remote_stream: &mut TcpStream,
    client_stream: &mut WebSocketStream<TcpStream>,
) -> Result<ProxyRequestResult, ProxyError> {
    //读取客户端请求
    let request_data = match read_request_result {
        Ok(option_data) => match option_data {
            Some(s) => s,
            None => {
                let error_message = "get empty data";
                let io_err = IoError::new(ErrorKind::Other, error_message);
                return Ok(ProxyRequestResult::Err(io_err));
            }
        },
        Err(e) => return Err(ProxyError::ws_err("read client request", e)),
    };
    //把请求发给远端
    let request_result = match remote_stream.write(&request_data).await {
        Ok(_) => ProxyRequestResult::Ok,
        Err(e) => {
            if e.kind() == ErrorKind::UnexpectedEof {
                ProxyRequestResult::Closed
            } else {
                ProxyRequestResult::Err(e)
            }
        }
    };
    let request_ret_msg = Message::from(&request_result);
    //把write远端的结果发给客户端
    if let Err(e) = client_stream.send(request_ret_msg).await {
        return Err(ProxyError::ws_err("write request result", e));
    }
    Ok(request_result)
}

///将响应转发给客户端
async fn proxy_send_response(
    read_response_result: Result<Vec<u8>, IoError>,
    client_stream: &mut WebSocketStream<TcpStream>,
) -> Result<ProxyResponseResult, ProxyError> {
    let response_result = match read_response_result {
        Ok(data) => ProxyResponseResult::Ok(data),
        Err(e) => {
            if e.kind() == ErrorKind::UnexpectedEof {
                ProxyResponseResult::Closed
            } else {
                ProxyResponseResult::Err(e)
            }
        }
    };
    let response_ret_msg = Message::from(&response_result);
    //把write远端的结果发给客户端
    if let Err(e) = client_stream.send(response_ret_msg).await {
        return Err(ProxyError::ws_err("write response to client", e));
    }
    Ok(response_result)
}
