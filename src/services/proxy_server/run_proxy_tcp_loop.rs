use super::io::ProxyConnectResult;
use super::proxy_error::ProxyError;
use super::proxy_tcp::proxy_tcp;
use super::wait_conn_remote::wait_conn_remote;
use crate::common::{socket5::ConnDest, ServerStream};
use crate::services::poll_message;
use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpStream, time::Duration};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

pub async fn run_proxy_tcp_loop(
    client_stream: WebSocketStream<ServerStream<TcpStream>>,
) -> Result<(), ProxyError> {
    let (mut client_writer, mut client_reader) = client_stream.split();
    loop {
        //读取客户端希望连接的地址、端口
        let conn_dest = match poll_message::poll_binary_message(&mut client_reader).await {
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
        println!("server connect {conn_dest}");
        let timeout_duration = Duration::from_secs(5);
        let conn_result = wait_conn_remote(&conn_dest, timeout_duration).await;
        let conn_ret_msg = Message::from(&conn_result);
        //把连接远端的结果发给客户端
        if let Err(e) = client_writer.send(conn_ret_msg).await {
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
        //proxy
        proxy_tcp(&mut client_reader, &mut client_writer, remote_stream).await?;
    }
}
