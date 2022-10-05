use super::proxy_error::ProxyError;
use super::{poll_message, send_message};
use crate::common::msg::server::{ProxyResponseResult, RequestFail};
use crate::common::msg::ClientMessage;
use crate::services::read_raw_data;
use actix_ws::{MessageStream, Session};
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
        let message = poll_message::poll_message(&mut session, client_reader).await?;
        let request_data = match message {
            ClientMessage::Request(s) => s.0,
            ClientMessage::DisConn => {
                //断开远端
                if let Err(e) = remote_writer.shutdown().await {
                    log::error!("disconn remote failed: {e}")
                };
                break;
            }
            _ => {
                //消息类型不对
                _ = session.close(None).await;
                return Err(ProxyError::NotRequestMessage);
            }
        };
        //把请求发给远端
        if let Err(e) = remote_writer.write(&request_data).await {
            let req_fail_msg = RequestFail(e.to_string());
            //把write远端的失败信息发给客户端
            send_message::send_message(&mut session, req_fail_msg).await?;
            //request失败跳出循环
            break;
        };
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
        let mut read_response_ok = true;
        let response_result_msg = match read_raw_data::read_raw(remote_reader).await {
            Ok(data) => ProxyResponseResult::Ok(data),
            Err(e) => {
                read_response_ok = false;
                if e.kind() == ErrorKind::UnexpectedEof {
                    ProxyResponseResult::Closed
                } else {
                    ProxyResponseResult::Err(e.to_string())
                }
            }
        };
        //把read远端的结果发给客户端
        send_message::send_message(&mut session, response_result_msg).await?;
        //response失败跳出循环
        if !read_response_ok {
            break;
        }
    }
    Ok(())
}
