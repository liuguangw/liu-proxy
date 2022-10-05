use super::poll_message;
use super::proxy_error::ProxyError;
use super::proxy_tcp::proxy_tcp;
use crate::{
    common::msg::{server::ConnectResult, ClientMessage},
    services::proxy_server::send_message,
};
use actix_ws::{MessageStream, Session};
use std::time::Duration;
use tokio::{net::TcpStream, time::timeout};

pub async fn run_proxy_tcp_loop(
    mut session: Session,
    mut msg_stream: MessageStream,
) -> Result<(), ProxyError> {
    loop {
        //读取客户端希望连接的地址、端口
        let message = poll_message::poll_message(&mut session, &mut msg_stream).await?;
        let conn_dest = match message {
            ClientMessage::Conn(s) => s.0,
            _ => {
                //消息类型不对
                _ = session.close(None).await;
                return Err(ProxyError::NotConnMessage);
            }
        };
        //指定超时时间, 执行connect
        log::info!("server connect {conn_dest}");
        let timeout_duration = Duration::from_secs(5);
        let (conn_result_msg, option_stream) =
            match timeout(timeout_duration, TcpStream::connect(&conn_dest)).await {
                Ok(inner_result) => match inner_result {
                    //成功
                    Ok(s) => (ConnectResult::Ok, Some(s)),
                    //失败
                    Err(e) => {
                        log::error!("server connect {conn_dest} failed: {e}");
                        (ConnectResult::Err(e.to_string()), None)
                    }
                },
                //超时
                Err(_) => {
                    log::error!("server connect {conn_dest} timeout");
                    (ConnectResult::Timeout, None)
                }
            };
        //把连接远端的结果发给客户端
        send_message::send_message(&mut session, conn_result_msg).await?;
        let remote_stream = match option_stream {
            Some(stream) => stream,
            None => continue,
        };
        //proxy
        proxy_tcp(session.to_owned(), &mut msg_stream, remote_stream).await?;
    }
}
