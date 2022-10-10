use super::proxy_error::ProxyError;
use super::{poll_message, send_message};
use crate::common::msg::server::{ProxyResponseResult, RequestFail};
use crate::common::msg::{ClientMessage, ServerMessage};
use crate::services::read_raw_data;
use axum::extract::ws::Message;
use axum::extract::ws::WebSocket;
use axum::Error as WsError;
use futures_util::{Sink, Stream, StreamExt};
use std::io::ErrorKind;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
};

pub async fn proxy_tcp(
    ws_stream: &mut WebSocket,
    mut remote_stream: TcpStream,
) -> Result<(), ProxyError> {
    let (remote_reader, remote_writer) = remote_stream.split();
    let (ws_writer, ws_reader) = ws_stream.split();
    let (tx, rx) = mpsc::channel(16);
    tokio::select! {
        request_result = read_request_loop(remote_writer,ws_reader, tx.clone()) => request_result,
        response_result = read_response_loop(remote_reader,tx) => response_result,
        send_result = send_message_loop(ws_writer,rx) => send_result,
    }
}

///将客户端请求转发给远端
async fn read_request_loop<T, U>(
    mut remote_writer: T,
    mut client_reader: U,
    tx: Sender<ServerMessage>,
) -> Result<(), ProxyError>
where
    T: AsyncWrite + Unpin,
    U: Stream<Item = Result<Message, WsError>> + Unpin,
{
    loop {
        //读取客户端请求
        let message = poll_message::poll_message(&mut client_reader).await?;
        let request_data = match message {
            ClientMessage::Request(s) => s.0,
            ClientMessage::DisConn => {
                //断开远端
                if let Err(e) = remote_writer.shutdown().await {
                    log::error!("disconn remote failed: {e}")
                };
                break;
            }
            //消息类型不对
            _ => return Err(ProxyError::NotRequestMessage),
        };
        //把请求发给远端
        if let Err(e) = remote_writer.write(&request_data).await {
            let req_fail_msg = RequestFail(e.to_string());
            //把write远端的失败信息发给客户端
            tx.send(req_fail_msg.into())
                .await
                .map_err(|_| ProxyError::ChannelSendMessage)?;
            //request失败跳出循环
            break;
        };
    }
    Ok(())
}

///将响应转发给客户端
async fn read_response_loop<T>(
    mut remote_reader: T,
    tx: Sender<ServerMessage>,
) -> Result<(), ProxyError>
where
    T: AsyncRead + Unpin,
{
    loop {
        let mut read_response_ok = true;
        let response_result_msg = match read_raw_data::read_raw(&mut remote_reader).await {
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
        tx.send(response_result_msg.into())
            .await
            .map_err(|_| ProxyError::ChannelSendMessage)?;
        //response失败跳出循环
        if !read_response_ok {
            break;
        }
    }
    Ok(())
}

async fn send_message_loop<T>(
    mut client_writer: T,
    mut rx: Receiver<ServerMessage>,
) -> Result<(), ProxyError>
where
    T: Sink<Message, Error = WsError> + Unpin,
{
    while let Some(msg) = rx.recv().await {
        //dbg!(&msg);
        send_message::send_message(&mut client_writer, msg)
            .await
            .map_err(ProxyError::SendMessage)?;
    }
    Ok(())
}
