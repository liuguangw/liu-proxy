use super::io::{ProxyRequestResult, ProxyResponseResult};
use super::proxy_error::ProxyError;
use crate::services::{poll_message, read_raw_data};
use futures_util::{SinkExt, StreamExt};
use std::io::ErrorKind;
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
};
use tokio_tungstenite::tungstenite::{Error as WsError, Message};

pub async fn proxy_tcp<T, U>(
    client_reader: &mut T,
    client_writer: &mut U,
    mut remote_stream: TcpStream,
) -> Result<(), ProxyError>
where
    T: StreamExt<Item = Result<Message, WsError>> + Unpin,
    U: SinkExt<Message, Error = WsError> + Unpin,
{
    let (mut remote_reader, mut remote_writer) = remote_stream.split();
    let (tx, rx) = mpsc::channel::<Message>(100);
    tokio::select! {
        request_result = read_request_loop(client_reader, &mut remote_writer, tx.clone())=>request_result,
        response_result = read_response_loop(&mut remote_reader, tx)=>response_result,
        write_result = write_client_loop(rx, client_writer) =>write_result
    }
}

///将客户端请求转发给远端
async fn read_request_loop<T, U>(
    client_reader: &mut T,
    remote_writer: &mut U,
    tx: Sender<Message>,
) -> Result<(), ProxyError>
where
    T: StreamExt<Item = Result<Message, WsError>> + Unpin,
    U: AsyncWrite + Unpin,
{
    loop {
        //读取客户端请求
        let request_data = match poll_message::poll_binary_message(client_reader).await {
            Ok(option_data) => match option_data {
                Some(s) => s,
                None => return Err(ProxyError::ClientClosed),
            },
            Err(e) => return Err(ProxyError::ws_err("read client request", e)),
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
        let request_ret_msg = Message::from(&request_result);
        tx.send(request_ret_msg).await?;
        //request失败跳出循环
        if !request_result.is_ok() {
            break;
        }
    }
    Ok(())
}

///将响应转发给客户端
async fn read_response_loop<T>(remote_reader: &mut T, tx: Sender<Message>) -> Result<(), ProxyError>
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
        //把write远端的结果发给客户端
        let response_ret_msg = Message::from(&response_result);
        tx.send(response_ret_msg).await?;
        //response失败跳出循环
        if !response_result.is_ok() {
            break;
        }
    }
    Ok(())
}

///把消息发送到客户端
async fn write_client_loop<T>(
    mut rx: Receiver<Message>,
    client_writer: &mut T,
) -> Result<(), ProxyError>
where
    T: SinkExt<Message, Error = WsError> + Unpin,
{
    while let Some(ret_msg) = rx.recv().await {
        //把write远端的结果发给客户端
        if let Err(e) = client_writer.send(ret_msg).await {
            return Err(ProxyError::ws_err("write client", e));
        }
    }
    Ok(())
}
