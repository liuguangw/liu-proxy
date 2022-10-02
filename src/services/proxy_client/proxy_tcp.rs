use super::io::ProxyRequestResult;
use super::io::ProxyResponseResult;
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

///将客户端请求转发给服务端
async fn proxy_send_request<T>(
    read_request_result: Result<Vec<u8>, IoError>,
    ws_stream: &mut T,
) -> Result<(), ProxyError>
where
    T: SinkExt<Message, Error = WsError> + Unpin,
{
    //读取客户端请求
    let request_data = match read_request_result {
        Ok(data) => data,
        Err(e) => {
            if e.kind() == ErrorKind::UnexpectedEof {
                return Err(ProxyError::ClientClosed);
            } else {
                return Err(ProxyError::io_err("read client request", e));
            }
        }
    };
    let request_msg = Message::binary(request_data);
    //把请求发给服务端
    if let Err(e) = ws_stream.send(request_msg).await {
        return Err(ProxyError::ws_err("send request to server", e));
    }
    Ok(())
}

///将响应转发给客户端
async fn proxy_send_response(
    read_response_result: Result<Option<Vec<u8>>, WsError>,
    tcp_stream: &mut TcpStream,
) -> Result<(), ProxyError> {
    let response_data = match read_response_result {
        Ok(option_data) => match option_data {
            Some(data) => data,
            None => return Err(ProxyError::ServerClosed),
        },
        Err(e) => return Err(ProxyError::ws_err("write response", e)),
    };
    let msg_type = response_data[0];
    if msg_type == 2 {
        //request ret
        let request_ret_result = ProxyRequestResult::from(&response_data[1..]);
        match request_ret_result {
            ProxyRequestResult::Ok => Ok(()),
            ProxyRequestResult::Err(e) => Err(ProxyError::RequestErr(e)),
            ProxyRequestResult::Closed => Err(ProxyError::RemoteClosed),
        }
    } else if msg_type == 3 {
        //response ret
        let response_ret_result = ProxyResponseResult::from(&response_data[1..]);
        let data = match response_ret_result {
            ProxyResponseResult::Ok(data) => data,
            ProxyResponseResult::Err(e) => return Err(ProxyError::ResponseErr(e)),
            ProxyResponseResult::Closed => return Err(ProxyError::RemoteClosed),
        };
        if let Err(err) = tcp_stream.write(&data).await {
            Err(ProxyError::io_err("write response", err))
        } else {
            Ok(())
        }
    } else {
        Err(ProxyError::InvalidRetMsgType(msg_type))
    }
}
