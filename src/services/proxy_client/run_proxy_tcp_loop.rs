use super::check_server_conn::check_server_conn;
use super::proxy_error::ProxyError;
use super::proxy_tcp::proxy_tcp;
use crate::common::socket5::{self, ConnDest};
use futures_util::{SinkExt, StreamExt};
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_tungstenite::tungstenite::{Error as WsError, Message};

pub async fn run_proxy_tcp_loop<T, U>(
    ws_reader: &mut T,
    ws_writer: &mut U,
    tcp_stream: &mut TcpStream,
    conn_dest: &ConnDest,
) -> Result<(), ProxyError>
where
    T: StreamExt<Item = Result<Message, WsError>> + Unpin,
    U: SinkExt<Message, Error = WsError> + Unpin,
{
    //把目标地址端口发给server,并检测server连接结果
    let rep = match check_server_conn(ws_reader, ws_writer, conn_dest).await {
        Ok(_) => 0,
        Err(e) => {
            println!("server conn {conn_dest} failed: {e}");
            if !e.is_ws_error() {
                //断开与server之间的连接
                if let Err(e1) = ws_writer.close().await {
                    println!("close conn failed: {e1}");
                }
            }
            5
        }
    };
    //写入socket5_response
    {
        let addr_raw_data = conn_dest.to_raw_data();
        let mut socket5_response = Vec::with_capacity(3 + addr_raw_data.len());
        socket5_response.push(socket5::VERSION);
        socket5_response.push(rep);
        socket5_response.push(0);
        socket5_response.extend_from_slice(&addr_raw_data);
        if let Err(e) = tcp_stream.write_all(&socket5_response).await {
            return Err(ProxyError::io_err("write socket5_response", e));
        }
    }
    if rep != 0 {
        return Ok(());
    }
    //println!("socket5 handshake success");
    // proxy
    proxy_tcp(ws_reader, ws_writer, tcp_stream).await
}
