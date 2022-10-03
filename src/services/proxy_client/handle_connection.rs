use super::proxy_error::ProxyError;
use super::proxy_handshake::proxy_handshake;
use super::run_proxy_tcp_loop::run_proxy_tcp_loop;
use crate::{
    common::{socket5::build_response, ClientConfig},
    services::proxy_client::auth_handshake,
};
use futures_util::{SinkExt, StreamExt};
use std::{net::SocketAddr, time::Duration};
use tokio::{io::AsyncWriteExt, net::TcpStream};

///处理连接逻辑
pub async fn handle_connection(mut stream: TcpStream, addr: SocketAddr, config: ClientConfig) {
    //socket5初步握手,获取目标地址,端口
    let conn_dest = match proxy_handshake(&mut stream).await {
        Ok(s) => s,
        Err(handshake_error) => {
            println!("socket5 handshake failed [{addr}]: {handshake_error}");
            return;
        }
    };
    //向服务端发起websocket连接,并进行认证
    let ws_stream = match auth_handshake(&config, Duration::from_secs(8)).await {
        Ok(s) => s.0,
        Err(e) => {
            eprintln!("{e}");
            //socket 5 通知失败信息
            let rep_code = 5;
            let socket5_response = build_response(&conn_dest, rep_code);
            if let Err(e1) = stream.write_all(&socket5_response).await {
                eprintln!("write socket5_response failed: {e1}");
            }
            return;
        }
    };
    let (mut ws_writer, mut ws_reader) = ws_stream.split();
    if let Err(proxy_error) =
        run_proxy_tcp_loop(&mut ws_reader, &mut ws_writer, &mut stream, &conn_dest).await
    {
        if !matches!(proxy_error, ProxyError::ClientClosed) {
            println!("{proxy_error}");
        }
        //断开与server之间的连接
        if !matches!(
            proxy_error,
            ProxyError::WsErr(_, _) | ProxyError::ServerClosed
        ) {
            if let Err(e1) = ws_writer.close().await {
                println!("close conn failed: {e1}");
            }
        }
    }
}
