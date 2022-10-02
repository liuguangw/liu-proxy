use super::io::ProxyConnectResult;
use super::proxy_error::ProxyError;
use super::proxy_tcp::proxy_tcp;
use super::wait_conn_remote::wait_conn_remote;
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
        //读取客户端希望连接的地址、端口
        let conn_dest = match poll_message::poll_binary_message(client_stream).await {
            Ok(option_data) => match option_data {
                Some(s) => ConnDest::try_from_bytes(&s)?,
                //客户端断开了连接
                None => return Err(ProxyError::ClientClosed),
            },
            Err(e) => {
                return Err(ProxyError::ws_err("get dest addr", e));
            }
        };
        //指定超时时间, 执行connect
        let timeout_duration = Duration::from_secs(5);
        let conn_result = wait_conn_remote(&conn_dest, timeout_duration).await;
        let conn_ret_msg = Message::from(&conn_result);
        //把连接远端的结果发给客户端
        if let Err(e) = client_stream.send(conn_ret_msg).await {
            return Err(ProxyError::ws_err("write conn result", e));
        }
        let remote_stream = match conn_result {
            ProxyConnectResult::Ok(stream) => stream,
            ProxyConnectResult::Err(e) => {
                println!("server connect {conn_dest} failed: {e}");
                continue;
            }
            ProxyConnectResult::Timeout => {
                println!("server connect {conn_dest} timeout");
                continue;
            }
        };
        //todo proxy
        proxy_tcp(client_stream, remote_stream).await?;
    }
}
