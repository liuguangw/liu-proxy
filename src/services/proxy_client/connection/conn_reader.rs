use super::{super::server_conn_manger::ConnPairReader, ConnectionError};
use crate::{
    common::msg::{server::ProxyResponseResult, ServerMessage},
    services::{
        proxy_client::poll_message::{self, PollMessageError},
        read_raw_data,
    },
};
use bytes::Bytes;
use futures_util::future::Either;
use std::io::ErrorKind;
use tokio::net::tcp::ReadHalf;

pub struct ConnReader<'a> {
    pub inner_reader: Either<ReadHalf<'a>, ConnPairReader>,
}

impl<'a> ConnReader<'a> {
    pub fn new(inner_reader: Either<ReadHalf<'a>, ConnPairReader>) -> Self {
        Self { inner_reader }
    }
    pub async fn read_data(&mut self) -> Result<Bytes, ConnectionError> {
        match &mut self.inner_reader {
            Either::Left(tcp_conn) => Self::tcp_read_data(tcp_conn).await,
            Either::Right(ws_conn) => Self::ws_read_data(ws_conn).await,
        }
    }
    async fn tcp_read_data(conn: &mut ReadHalf<'_>) -> Result<Bytes, ConnectionError> {
        match read_raw_data::read_raw(conn).await {
            Ok(data) => Ok(data),
            Err(e) => {
                let err_kind = e.kind();
                if err_kind == ErrorKind::UnexpectedEof || err_kind == ErrorKind::ConnectionAborted
                {
                    Err(ConnectionError::ConnClosed)
                } else {
                    //dbg!(&e);
                    Err(ConnectionError::TcpRead(e))
                }
            }
        }
    }
    async fn ws_read_data(conn: &mut ConnPairReader) -> Result<Bytes, ConnectionError> {
        let message = match poll_message::poll_message(conn).await {
            Ok(s) => s,
            Err(e) => match e {
                PollMessageError::Closed => return Err(ConnectionError::ConnClosed),
                PollMessageError::Protocol(e1) => return Err(ConnectionError::WsRead(e1)),
                PollMessageError::Parse(e2) => return Err(ConnectionError::WsParseMsg(e2)),
            },
        };
        match message {
            //类型错误, 此时不应该收到这种消息
            ServerMessage::ConnResult(_) => Err(ConnectionError::WsInvalidServerMessage),
            ServerMessage::ResponseResult(response_result) => match response_result {
                //得到response
                ProxyResponseResult::Ok(response_data) => Ok(response_data),
                //server读取response失败
                ProxyResponseResult::Err(e) => Err(ConnectionError::WsServerResponse(e)),
                //远端关闭了与server之间的连接
                ProxyResponseResult::Closed => Err(ConnectionError::WsRemoteClosed),
            },
            //server发送request到远端失败
            ServerMessage::RequestFail(e) => Err(ConnectionError::WsServerRequest(e.0)),
        }
    }
}
