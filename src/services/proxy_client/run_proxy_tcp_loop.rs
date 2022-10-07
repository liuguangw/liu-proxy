use super::proxy_error::ProxyError;
use super::proxy_tcp::proxy_tcp;
use super::server_conn_manger::ConnPair;
use super::{check_server_conn::check_server_conn, server_conn_manger::ServerConnManger};
use crate::common::socks5::{build_response, ConnDest};
use tokio::{io::AsyncWriteExt, net::TcpStream};

pub async fn run_proxy_tcp_loop(
    conn_manger: ServerConnManger,
    mut ws_conn_pair: ConnPair,
    tcp_stream: &mut TcpStream,
    conn_dest: &ConnDest,
) -> Result<(), ProxyError> {
    //
    let mut is_ws_err = false;
    //把目标地址端口发给server,并检测server连接结果
    let rep = match check_server_conn(&mut ws_conn_pair, conn_dest).await {
        Ok(_) => {
            log::info!("server conn {conn_dest} ok");
            0
        }
        Err(e) => {
            log::error!("server conn {conn_dest} failed: {e}");
            is_ws_err = e.is_ws_error();
            5
        }
    };
    //写入socks5_response
    {
        let socks5_response = build_response(conn_dest, rep);
        if let Err(e) = tcp_stream.write_all(&socks5_response).await {
            //回收连接
            if !is_ws_err {
                conn_manger.push_back_conn(ws_conn_pair).await;
            }
            return Err(ProxyError::Socks5Resp(e));
        }
    }
    //server连接remote失败
    if rep != 0 {
        //回收连接
        if !is_ws_err {
            conn_manger.push_back_conn(ws_conn_pair).await;
        }
        return Ok(());
    }
    //println!("socks5 handshake success");
    // proxy
    let proxy_result = proxy_tcp(&mut ws_conn_pair, tcp_stream).await;
    let is_ws_err = match &proxy_result {
        Ok(_) => false,
        Err(e) => e.is_ws_error(),
    };
    //回收连接
    if !is_ws_err {
        conn_manger.push_back_conn(ws_conn_pair).await;
    }
    proxy_result
}
