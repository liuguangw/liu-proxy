use super::io::ProxyConnectResult;
use super::poll_message;
use super::proxy_error::ProxyError;
use super::proxy_tcp::proxy_tcp;
use super::wait_conn_remote::wait_conn_remote;
use crate::common::socket5::ConnDest;
use actix_ws::{MessageStream, Session};
use std::time::Duration;

pub async fn run_proxy_tcp_loop(
    mut session: Session,
    mut msg_stream: MessageStream,
) -> Result<(), ProxyError> {
    loop {
        //读取客户端希望连接的地址、端口
        let conn_dest = match poll_message::poll_binary_message(&mut session, &mut msg_stream).await
        {
            Some(msg_result) => match msg_result {
                Ok(bytes) => match ConnDest::try_from_bytes(&bytes) {
                    Ok(s) => s,
                    Err(e) => {
                        //目标地址信息解析失败,关闭连接
                        _ = session.close(None).await;
                        return Err(e.into());
                    }
                },
                Err(e) => return Err(ProxyError::ws_err("get dest addr", e)),
            },
            //客户端断开了连接
            None => return Err(ProxyError::ClientClosed),
        };
        //指定超时时间, 执行connect
        log::info!("server connect {conn_dest}");
        let timeout_duration = Duration::from_secs(5);
        let conn_result = wait_conn_remote(&conn_dest, timeout_duration).await;
        let conn_ret_msg = conn_result.to_bytes();
        //把连接远端的结果发给客户端
        if session.binary(conn_ret_msg).await.is_err() {
            return Err(ProxyError::ClientClosed);
        }
        let remote_stream = match conn_result {
            ProxyConnectResult::Ok(stream) => stream,
            ProxyConnectResult::Err(e) => {
                log::error!("server connect {conn_dest} failed: {e}");
                continue;
            }
            ProxyConnectResult::Timeout => {
                log::error!("server connect {conn_dest} timeout");
                continue;
            }
        };
        //proxy
        proxy_tcp(session.to_owned(), &mut msg_stream, remote_stream).await?;
    }
}
