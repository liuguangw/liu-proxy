use crate::{
    common::{RouteConfigAction, RouteConfigCom},
    services::proxy_client::{
        check_server_conn,
        server_conn_manger::{ConnPair, ServerConnManger},
    },
};
use futures_util::future::Either;
use tokio::net::TcpStream;

use super::{conn_reader::ConnReader, conn_writer::ConnWriter, ConnectionError};

///代表一个连接server的连接,或者是直连的连接
pub struct RemoteConnection {
    conn: Either<TcpStream, Option<ConnPair>>,
}

impl RemoteConnection {
    pub async fn connect(
        conn_dest: &str,
        conn_manger: &ServerConnManger,
        route_config: &RouteConfigCom,
    ) -> Result<Self, ConnectionError> {
        let t_action = route_config.match_action(conn_dest);
        log::info!("[{t_action:?}]{conn_dest}");
        let conn = match t_action {
            RouteConfigAction::Direct => Either::Left(Self::conn_direct(conn_dest).await?),
            RouteConfigAction::Proxy => {
                let ws_conn = Self::conn_server(conn_dest, conn_manger).await?;
                Either::Right(Some(ws_conn))
            }
            RouteConfigAction::Block => return Err(ConnectionError::RouteBlocked),
        };
        Ok(Self { conn })
    }

    ///连接websocket server
    async fn conn_server(
        conn_dest: &str,
        conn_manger: &ServerConnManger,
    ) -> Result<ConnPair, ConnectionError> {
        let mut ws_conn_pair = conn_manger
            .get_conn_pair()
            .await
            .map_err(ConnectionError::WsConn)?;
        //把目标地址端口发给server,并检测server连接结果
        match check_server_conn::check_server_conn(&mut ws_conn_pair, &conn_dest).await {
            Ok(_) => {
                //log::info!("[Proxy]{conn_dest} conn ok");
                Ok(ws_conn_pair)
            }
            Err(e) => {
                //log::info!("[Proxy]{conn_dest} conn failed: {e}");
                //回收连接
                if !e.is_ws_error() {
                    conn_manger.push_back_conn(ws_conn_pair).await;
                }
                Err(ConnectionError::ServerConn(e))
            }
        }
    }
    ///直连
    async fn conn_direct(conn_dest: &str) -> Result<TcpStream, ConnectionError> {
        TcpStream::connect(conn_dest)
            .await
            .map_err(|e| ConnectionError::TcpConn(conn_dest.to_string(), e))
    }

    ///分割为writer和reader
    pub fn split(&mut self) -> (ConnWriter, ConnReader) {
        match &mut self.conn {
            Either::Left(tcp_conn) => {
                let (reader, writer) = tcp_conn.split();
                (
                    ConnWriter::new(Either::Left(writer)),
                    ConnReader::new(Either::Left(reader)),
                )
            }
            Either::Right(option_ws_conn) => {
                let (writer, reader) = option_ws_conn.take().unwrap();
                (
                    ConnWriter::new(Either::Right(writer)),
                    ConnReader::new(Either::Right(reader)),
                )
            }
        }
    }
    pub async fn push_back_conn(self, conn_manger: &ServerConnManger) {
        if let Either::Right(Some(ws_conn)) = self.conn {
            conn_manger.push_back_conn(ws_conn).await;
        }
    }
}
