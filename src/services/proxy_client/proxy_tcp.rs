use super::poll_message;
use super::proxy_error::ProxyError;
use super::server_conn_manger::ConnPair;
use super::stream::{StreamReader, StreamWriter};
use crate::common::msg::client::ProxyRequest;
use crate::common::msg::{server::ProxyResponseResult, ClientMessage, ServerMessage};
use futures_util::future::Either;
use futures_util::{Sink, Stream};
use std::io::ErrorKind;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};

pub async fn proxy_tcp(
    ws_conn_pair: &mut ConnPair,
    stream_reader: &mut StreamReader<'_>,
    stream_writer: &mut StreamWriter<'_>,
) -> Result<(), ProxyError> {
    tokio::select! {
        request_result = read_request_loop(stream_reader, &mut ws_conn_pair.0)=>request_result,
        response_result = read_response_loop(&mut ws_conn_pair.1, stream_writer)=>response_result,
    }
}

///将客户端请求转发给server
async fn read_request_loop<T>(
    stream_reader: &mut StreamReader<'_>,
    ws_writer: &mut T,
) -> Result<(), ProxyError>
where
    T: Sink<Message, Error = WsError> + Unpin,
{
    loop {
        let mut is_disconn = false;
        //读取客户端请求
        let client2_msg = match stream_reader.read_buf().await {
            //客户端的请求
            Ok(data) => Either::Left(ProxyRequest(data)),
            Err(e) => {
                //客户端断开socks5
                if e.kind() == ErrorKind::UnexpectedEof {
                    is_disconn = true;
                    Either::Right(ClientMessage::DisConn)
                } else {
                    return Err(ProxyError::ReadRequest(e));
                }
            }
        };
        //把请求发给服务端
        let send_result = match client2_msg {
            Either::Left(request_msg) => {
                super::send_message::send_message(ws_writer, request_msg).await
            }
            Either::Right(disconn_msg) => {
                super::send_message::send_message(ws_writer, disconn_msg).await
            }
        };
        send_result.map_err(ProxyError::SendRequest)?;
        if is_disconn {
            break;
        }
    }
    Ok(())
}

///将响应给客户端
async fn read_response_loop<T>(
    ws_reader: &mut T,
    stream_writer: &mut StreamWriter<'_>,
) -> Result<(), ProxyError>
where
    T: Stream<Item = Result<Message, WsError>> + Unpin,
{
    loop {
        let message = poll_message::poll_message(ws_reader)
            .await
            .map_err(ProxyError::PollMessage)?;
        let response_data = match message {
            //类型错误, 此时不应该收到这种消息
            ServerMessage::ConnResult(_) => return Err(ProxyError::InvalidServerMessage),
            ServerMessage::ResponseResult(response_result) => match response_result {
                //得到response
                ProxyResponseResult::Ok(s) => s,
                //server读取response失败
                ProxyResponseResult::Err(e) => return Err(ProxyError::ServerResponse(e)),
                //远端关闭了与server之间的连接
                ProxyResponseResult::Closed => break,
            },
            //server发送request到远端失败
            ServerMessage::RequestFail(e) => return Err(ProxyError::ServerRequest(e.0)),
        };
        //dbg!(&response_data);
        if let Err(e) = stream_writer.write_buf(response_data).await {
            return Err(ProxyError::WriteResponse(e));
        }
    }
    Ok(())
}
