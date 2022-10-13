use super::proxy_error::ProxyError;
use crate::common::msg::{ClientMessage, ServerMessage};
use axum::extract::ws::{Message, WebSocket};
use bytes::Bytes;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::sync::mpsc::{Receiver, Sender};

///读取客户端消息
pub async fn read_from_client(
    mut reader: SplitStream<WebSocket>,
    tx: Sender<ClientMessage>,
) -> Result<(), ProxyError> {
    while let Some(message_result) = reader.next().await {
        let message = message_result.map_err(ProxyError::ReadClient)?;
        if let Message::Binary(data) = message {
            //解析消息
            let data_bytes = Bytes::from(data);
            let client_message = ClientMessage::try_from(data_bytes)?;
            //发送给处理程序
            tx.send(client_message)
                .await
                .map_err(|_| ProxyError::ReadChannel)?;
        }
    }
    Ok(())
}

///将服务端消息发给客户端
pub async fn write_to_client(
    mut writer: SplitSink<WebSocket, Message>,
    mut rx: Receiver<ServerMessage>,
) -> Result<(), ProxyError> {
    while let Some(server_msg) = rx.recv().await {
        let msg_bytes: Bytes = server_msg.into();
        let message = Message::Binary(msg_bytes.to_vec());
        writer
            .send(message)
            .await
            .map_err(ProxyError::WriteClient)?;
    }
    Ok(())
}
