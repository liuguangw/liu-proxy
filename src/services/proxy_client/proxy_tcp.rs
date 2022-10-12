use super::poll_message;
use super::proxy_error::ProxyError;
use super::server_conn_manger::ConnPair;
use crate::common::msg::{
    client::ProxyRequest, server::ProxyResponseResult, ClientMessage, ServerMessage,
};
use crate::services::read_raw_data;
use futures_util::{Sink, Stream};
use std::io::ErrorKind;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};

pub async fn proxy_tcp(
    ws_conn_pair: &mut ConnPair,
    stream: &mut TcpStream,
) -> Result<(), ProxyError> {
    let (mut stream_reader, mut stream_writer) = stream.split();
    tokio::select! {
        request_result = read_request_loop(&mut stream_reader, &mut ws_conn_pair.0)=>request_result,
        response_result = read_response_loop(&mut ws_conn_pair.1,&mut  stream_writer)=>response_result,
    }
}

///将客户端请求转发给server
async fn read_request_loop<T, U>(stream_reader: &mut T, ws_writer: &mut U) -> Result<(), ProxyError>
where
    T: AsyncRead + Unpin,
    U: Sink<Message, Error = WsError> + Unpin,
{
    loop {
        //读取客户端请求
        let raw_data = match read_raw_data::read_raw(stream_reader).await {
            Ok(data) => data,
            Err(e) => {
                //proxy被断开,通知服务端断开remote
                let disconn_msg = ClientMessage::DisConn;
                super::send_message::send_message(ws_writer, disconn_msg)
                    .await
                    .map_err(ProxyError::SendRequest)?;
                let err_kind = e.kind();
                if err_kind == ErrorKind::UnexpectedEof || err_kind == ErrorKind::ConnectionAborted
                {
                    //被主动断开
                    break;
                } else {
                    //因为读取错误而断开
                    //dbg!(&e);
                    return Err(ProxyError::ReadRequest(e));
                }
            }
        };
        let request_msg = ProxyRequest(raw_data);
        //把请求发给服务端
        super::send_message::send_message(ws_writer, request_msg)
            .await
            .map_err(ProxyError::SendRequest)?;
    }
    Ok(())
}

///将响应给客户端
async fn read_response_loop<T, U>(
    ws_reader: &mut T,
    stream_writer: &mut U,
) -> Result<(), ProxyError>
where
    T: Stream<Item = Result<Message, WsError>> + Unpin,
    U: AsyncWrite + Unpin,
{
    loop {
        let message = poll_message::poll_message(ws_reader)
            .await
            .map_err(ProxyError::PollMessage)?;
        let mut response_data = match message {
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
        if let Err(e) = stream_writer.write_all_buf(&mut response_data).await {
            return Err(ProxyError::WriteResponse(e));
        }
    }
    Ok(())
}
