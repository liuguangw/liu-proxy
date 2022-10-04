use super::io::{ProxyRequestResult, ProxyResponseResult};
use super::poll_message;
use super::proxy_error::ProxyError;
use crate::services::read_raw_data;
use actix_ws::{MessageStream, Session};
use bytes::Bytes;
use std::io::ErrorKind;
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};

pub async fn proxy_tcp(
    session: Session,
    msg_stream: &mut MessageStream,
    mut remote_stream: TcpStream,
) -> Result<(), ProxyError> {
    let (mut remote_reader, mut remote_writer) = remote_stream.split();
    tokio::select! {
        request_result = read_request_loop(session.to_owned(), msg_stream, &mut remote_writer)=>request_result,
        response_result = read_response_loop(session, &mut remote_reader)=>response_result,
    }
}

///将客户端请求转发给远端
async fn read_request_loop<T>(
    mut session: Session,
    client_reader: &mut MessageStream,
    remote_writer: &mut T,
) -> Result<(), ProxyError>
where
    T: AsyncWrite + Unpin,
{
    loop {
        //读取客户端请求
        let request_data =
            match poll_message::poll_binary_message(&mut session, client_reader).await {
                Some(msg_result) => match msg_result {
                    Ok(s) => s,
                    Err(e) => return Err(ProxyError::ws_err("read client request", e)),
                },
                None => return Err(ProxyError::ClientClosed),
            };
        //把请求发给远端
        let request_result = match remote_writer.write(&request_data).await {
            Ok(_) => ProxyRequestResult::Ok,
            Err(e) => {
                if e.kind() == ErrorKind::UnexpectedEof {
                    ProxyRequestResult::Closed
                } else {
                    ProxyRequestResult::Err(e)
                }
            }
        };
        //把write远端的结果发给客户端
        write_client_msg(&mut session, request_result.to_bytes()).await?;
        //request失败跳出循环
        if !request_result.is_ok() {
            break;
        }
    }
    Ok(())
}

///将响应转发给客户端
async fn read_response_loop<T>(
    mut session: Session,
    remote_reader: &mut T,
) -> Result<(), ProxyError>
where
    T: AsyncRead + Unpin,
{
    loop {
        let response_result = match read_raw_data::read_raw(remote_reader).await {
            Ok(data) => ProxyResponseResult::Ok(data),
            Err(e) => {
                if e.kind() == ErrorKind::UnexpectedEof {
                    ProxyResponseResult::Closed
                } else {
                    ProxyResponseResult::Err(e)
                }
            }
        };
        //把read远端的结果发给客户端
        write_client_msg(&mut session, response_result.to_bytes()).await?;
        //response失败跳出循环
        if !response_result.is_ok() {
            break;
        }
    }
    Ok(())
}

///把消息发送到客户端
async fn write_client_msg(session: &mut Session, msg: Bytes) -> Result<(), ProxyError> {
    if session.binary(msg).await.is_err() {
        return Err(ProxyError::ClientClosed);
    }
    Ok(())
}
