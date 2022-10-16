use super::{super::server_conn_manger::ConnPairWriter, ConnectionError};
use crate::{
    common::msg::{client::ProxyRequest, ClientMessage},
    services::proxy_client::send_message,
};
use bytes::Bytes;
use futures_util::future::Either;
use tokio::{io::AsyncWriteExt, net::tcp::WriteHalf};

pub struct ConnWriter<'a> {
    pub inner_writer: Either<WriteHalf<'a>, ConnPairWriter>,
}

impl<'a> ConnWriter<'a> {
    pub fn new(inner_writer: Either<WriteHalf<'a>, ConnPairWriter>) -> Self {
        Self { inner_writer }
    }
    pub async fn write_data(&mut self, mut data: Bytes) -> Result<(), ConnectionError> {
        match &mut self.inner_writer {
            Either::Left(tcp_writer) => {
                tcp_writer
                    .write_all_buf(&mut data)
                    .await
                    .map_err(ConnectionError::TcpWrite)?;
            }
            Either::Right(ws_writer) => {
                let request_msg = ProxyRequest(data);
                send_message::send_message(ws_writer, request_msg)
                    .await
                    .map_err(ConnectionError::WsWrite)?;
            }
        };
        Ok(())
    }

    pub async fn process_client_close(&mut self) -> Result<(), ConnectionError> {
        match &mut self.inner_writer {
            Either::Left(tcp_writer) => {
                _ = tcp_writer.shutdown().await;
            }
            Either::Right(ws_writer) => {
                //proxy被断开,通知服务端断开remote
                let disconn_msg = ClientMessage::DisConn;
                send_message::send_message(ws_writer, disconn_msg)
                    .await
                    .map_err(ConnectionError::WsWrite)?;
            }
        };
        Ok(())
    }
}
