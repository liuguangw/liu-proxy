use super::proxy_error::ProxyError;
use super::proxy_tcp::proxy_tcp;
use super::wait_conn_remote::wait_conn_remote;
use super::wait_conn_remote::ConnRemoteError;
use crate::common::socket5::ConnDest;
use crate::services::poll_message;
use futures_util::SinkExt;
use tokio::net::TcpStream;
use tokio::time::Duration;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

pub async fn run_proxy_tcp_loop(
    client_stream: &mut WebSocketStream<TcpStream>,
) -> Result<(), ProxyError> {
    loop {
        let conn_dest = match poll_message::poll_binary_message(client_stream).await {
            Ok(s) => {
                ConnDest::try_from_bytes(&s)?
            }
            Err(e) => {
                return Err(ProxyError::ws_err("get dest addr", e));
            }
        };
        let timeout_duration = Duration::from_secs(5);
        let (stream, conn_ret_msg) = match wait_conn_remote(&conn_dest, timeout_duration).await {
            Ok(s) => {
                //连接remote成功
                let msg = Message::binary(vec![0]);
                (Some(s), msg)
            }
            Err(err) => {
                let error_message = err.to_string();
                let mut data = Vec::with_capacity(1 + error_message.len());
                let ret_code = match err {
                    //出错
                    ConnRemoteError::Err(_) =>1,
                    //超时
                    ConnRemoteError::Timeout =>2
                };
                data.push(ret_code);
                data.extend_from_slice(error_message.as_bytes());
                let msg = Message::binary(data);
                (None, msg)
            }
        };
        //把连接远端的结果发给客户端
        if let Err(e) = client_stream.send(conn_ret_msg).await {
            return Err(ProxyError::ws_err("write conn result", e));
        }
        let remote_stream = match stream {
            Some(s) => s,
            //进入下一轮循环
            None => continue,
        };
        //todo proxy
        proxy_tcp(client_stream, remote_stream).await?;
    }
}
