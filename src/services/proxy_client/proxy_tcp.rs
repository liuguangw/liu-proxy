use super::io::{ProxyRequestResult, ProxyResponseResult};
use super::poll_message;
use super::proxy_error::ProxyError;
use crate::services::read_raw_data;
use futures_util::{SinkExt, StreamExt};
use std::io::ErrorKind;
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};
use tokio_tungstenite::tungstenite::{Error as WsError, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub async fn proxy_tcp(
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    tcp_stream: &mut TcpStream,
) -> Result<(), ProxyError> {
    let (mut tcp_reader, mut tcp_writer) = tcp_stream.split();
    let (mut ws_writer, mut ws_reader) = ws_stream.split();
    tokio::select! {
        request_result = read_request_loop(&mut tcp_reader, &mut ws_writer)=>request_result,
        response_result = read_response_loop(&mut ws_reader, &mut tcp_writer)=>response_result,
    }
}

///将客户端请求转发给server
async fn read_request_loop<T, U>(tcp_reader: &mut T, ws_writer: &mut U) -> Result<(), ProxyError>
where
    T: AsyncRead + Unpin,
    U: SinkExt<Message, Error = WsError> + Unpin,
{
    loop {
        //读取客户端请求
        let request_data = match read_raw_data::read_raw(tcp_reader).await {
            Ok(data) => data,
            Err(e) => {
                if e.kind() == ErrorKind::UnexpectedEof {
                    return Err(ProxyError::ClientClosed);
                } else {
                    return Err(ProxyError::io_err("read client request", e));
                }
            }
        };
        //把请求发给服务端
        let request_msg = Message::binary(request_data);
        if let Err(e) = ws_writer.send(request_msg).await {
            return Err(ProxyError::ws_err("send request to server", e));
        }
    }
}

///将响应给客户端
async fn read_response_loop<T, U>(ws_reader: &mut T, tcp_writer: &mut U) -> Result<(), ProxyError>
where
    T: StreamExt<Item = Result<Message, WsError>> + Unpin,
    U: AsyncWrite + Unpin,
{
    loop {
        let response_data = match poll_message::poll_binary_message(ws_reader).await {
            Ok(option_data) => match option_data {
                Some(data) => data,
                None => return Err(ProxyError::ServerClosed),
            },
            Err(e) => return Err(ProxyError::ws_err("read server response", e)),
        };
        let msg_type = response_data[0];
        if msg_type == 2 {
            //request ret
            let request_ret_result = ProxyRequestResult::from(&response_data[1..]);
            match request_ret_result {
                ProxyRequestResult::Ok => (),
                ProxyRequestResult::Err(e) => return Err(ProxyError::RequestErr(e)),
                ProxyRequestResult::Closed => return Err(ProxyError::RemoteClosed),
            };
        } else if msg_type == 3 {
            //response ret
            let response_ret_result = ProxyResponseResult::from(&response_data[1..]);
            let data = match response_ret_result {
                ProxyResponseResult::Ok(data) => data,
                ProxyResponseResult::Err(e) => return Err(ProxyError::ResponseErr(e)),
                ProxyResponseResult::Closed => return Err(ProxyError::RemoteClosed),
            };
            if let Err(err) = tcp_writer.write(&data).await {
                return Err(ProxyError::io_err("write response", err));
            }
        } else {
            return Err(ProxyError::InvalidRetMsgType(msg_type));
        }
    }
}
