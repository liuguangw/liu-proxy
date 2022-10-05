use super::check_server_conn::check_server_conn;
use super::proxy_error::ProxyError;
use super::proxy_tcp::proxy_tcp;
use crate::common::socket5::{build_response, ConnDest};
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub async fn run_proxy_tcp_loop(
    mut ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    tcp_stream: &mut TcpStream,
    conn_dest: &ConnDest,
) -> Result<(), ProxyError> {
    //把目标地址端口发给server,并检测server连接结果
    let rep = match check_server_conn(&mut ws_stream, conn_dest).await {
        Ok(_) => {
            log::info!("server conn {conn_dest} ok");
            0
        }
        Err(e) => {
            log::error!("server conn {conn_dest} failed: {e}");
            if !e.is_ws_error() {
                //断开与server之间的连接
                if let Err(e1) = ws_stream.close(None).await {
                    log::error!("close conn failed: {e1}");
                }
            }
            5
        }
    };
    //写入socket5_response
    {
        let socket5_response = build_response(conn_dest, rep);
        if let Err(e) = tcp_stream.write_all(&socket5_response).await {
            //断开与server之间的连接
            if rep == 0 {
                if let Err(e1) = ws_stream.close(None).await {
                    log::error!("close conn failed: {e1}");
                }
            }
            return Err(ProxyError::Socket5Resp(e));
        }
    }
    //server连接remote失败
    if rep != 0 {
        return Ok(());
    }
    //println!("socket5 handshake success");
    // proxy
    proxy_tcp(ws_stream, tcp_stream).await
}
