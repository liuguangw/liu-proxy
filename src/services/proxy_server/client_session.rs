use super::{client_io, proxy_error::ProxyError, read_remote_stream};
use crate::common::msg::{
    client::{Connect, ProxyRequest},
    server::{ConnectResult, RequestFail},
    ClientMessage, ServerMessage,
};
use axum::extract::ws::WebSocket;
use futures_util::StreamExt;
use std::time::Duration;
use tokio::{
    io::AsyncWriteExt,
    net::{tcp::WriteHalf, TcpStream},
    sync::mpsc::{self, Receiver, Sender},
    time,
};

pub struct ClientSession {
    pub username: String,
    ///连接复用次数
    pub use_count: u8,
}

impl ClientSession {
    pub fn new(username: String) -> Self {
        Self {
            username,
            use_count: 0,
        }
    }
    pub async fn run_proxy(&mut self, ws_stream: WebSocket) -> Result<(), ProxyError> {
        let (tx1, rx1) = mpsc::channel(20);
        let (tx2, rx2) = mpsc::channel(20);
        let (writer, reader) = ws_stream.split();
        tokio::select! {
            proc_result = self.process_message(tx2,rx1)=>proc_result,
            read_result = client_io::read_from_client(reader, tx1)=>read_result,
            write_result = client_io::write_to_client(writer, rx2)=>write_result,
        }
    }
    ///处理客户端消息
    pub async fn process_message(
        &mut self,
        tx: Sender<ServerMessage>,
        mut rx: Receiver<ClientMessage>,
    ) -> Result<(), ProxyError> {
        while let Some(message) = rx.recv().await {
            //外围只处理Conn
            if let ClientMessage::Conn(conn_msg) = message {
                let mut option_conn_msg = self.process_conn(conn_msg, tx.clone(), &mut rx).await?;
                while let Some(conn_msg) = option_conn_msg {
                    option_conn_msg = self.process_conn(conn_msg, tx.clone(), &mut rx).await?;
                }
            }
        }
        Ok(())
    }

    async fn process_conn(
        &mut self,
        conn_msg: Connect,
        tx: Sender<ServerMessage>,
        rx: &mut Receiver<ClientMessage>,
    ) -> Result<Option<Connect>, ProxyError> {
        //log::info!("ClientMessage::Conn");
        let conn_dest = conn_msg.0;
        //指定超时时间, 执行connect
        log::info!("[{}]server connect {conn_dest}", self.username);
        let timeout_duration = Duration::from_secs(5);
        let (conn_result_msg, option_stream) =
            match time::timeout(timeout_duration, TcpStream::connect(&conn_dest)).await {
                Ok(inner_result) => match inner_result {
                    //成功
                    Ok(s) => (ConnectResult::Ok, Some(s)),
                    //失败
                    Err(e) => {
                        log::error!("[{}]server connect {conn_dest} failed: {e}", self.username);
                        (ConnectResult::Err(e.to_string()), None)
                    }
                },
                //超时
                Err(_) => {
                    log::error!("[{}]server connect {conn_dest} timeout", self.username);
                    (ConnectResult::Timeout, None)
                }
            };
        //向客户端发送连接结果
        tx.send(conn_result_msg.into())
            .await
            .map_err(|_| ProxyError::WriteChannel)?;
        if let Some(remote_stream) = option_stream {
            //复用计数+1
            self.use_count += 1;
            log::info!(
                "[{}]server connect {conn_dest} ok (#{})",
                self.username,
                self.use_count
            );
            //处理远程连接
            let option_conn_msg = self.process_remote(remote_stream, tx, rx).await?;
            return Ok(option_conn_msg);
        }
        Ok(None)
    }

    //处理远程连接io
    async fn process_remote(
        &mut self,
        mut remote_stream: TcpStream,
        tx: Sender<ServerMessage>,
        rx: &mut Receiver<ClientMessage>,
    ) -> Result<Option<Connect>, ProxyError> {
        let (remote_reader, remote_writer) = remote_stream.split();
        //处理客户端消息和读取远端一起运行
        let option_conn_msg = tokio::select! {
            client_result = self.process_client_message(remote_writer,tx.clone(),rx)=>{
                //dbg!(&client_result);
                client_result?
            },
            remote_result = read_remote_stream::read_remote_stream(remote_reader,tx)=>{
                //dbg!(&remote_result);
                remote_result?;
                None
            },
        };
        //关闭远端连接
        _ = remote_stream.shutdown().await;
        Ok(option_conn_msg)
    }

    ///处理客户端的所有消息(remote已连接的条件下)
    async fn process_client_message(
        &mut self,
        mut remote_writer: WriteHalf<'_>,
        mut tx: Sender<ServerMessage>,
        rx: &mut Receiver<ClientMessage>,
    ) -> Result<Option<Connect>, ProxyError> {
        while let Some(message) = rx.recv().await {
            match message {
                ClientMessage::Conn(conn_msg) => {
                    //未收到断开消息,就又conn的异常情况
                    //log::info!("ClientMessage::Conn");
                    log::warn!("conn new remote without send ClientMessage::DisConn");
                    return Ok(Some(conn_msg));
                }
                ClientMessage::DisConn => {
                    //log::info!("ClientMessage::DisConn");
                    break;
                }
                ClientMessage::Request(req_msg) => {
                    //log::info!("ClientMessage::Request");
                    self.process_request(&mut remote_writer, &mut tx, req_msg)
                        .await?
                }
            }
        }
        Ok(None)
    }

    ///把请求数据发给远端
    async fn process_request(
        &mut self,
        remote_writer: &mut WriteHalf<'_>,
        tx: &mut Sender<ServerMessage>,
        req_msg: ProxyRequest,
    ) -> Result<(), ProxyError> {
        let request_data = req_msg.0;
        if let Err(e) = remote_writer.write_all(&request_data).await {
            log::error!("write remote failed: {e}");
            //把write远端的失败信息发给客户端
            let req_fail_msg = RequestFail(e.to_string());
            tx.send(req_fail_msg.into())
                .await
                .map_err(|_| ProxyError::WriteChannel)?;
        }
        Ok(())
    }
}
