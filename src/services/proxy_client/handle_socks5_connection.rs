use super::check_server_conn::check_server_conn;
use super::run_proxy_tcp_loop::run_proxy_tcp_loop;
use super::stream::{StreamReader, StreamWriter};
use super::{proxy_handshake::proxy_handshake, server_conn_manger::ServerConnManger};
use crate::common::socks5::build_response;
use std::net::SocketAddr;
use tokio::{io::AsyncWriteExt, net::TcpStream};

///处理连接逻辑
pub async fn handle_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    conn_manger: ServerConnManger,
) {
    //socks5初步握手,获取目标地址,端口
    let conn_dest = match proxy_handshake(&mut stream).await {
        Ok(s) => s,
        Err(handshake_error) => {
            log::error!("socks5 handshake failed [{addr}]: {handshake_error}");
            return;
        }
    };
    //向服务端发起websocket连接,并进行认证
    let mut ws_conn_pair = match conn_manger.get_conn_pair().await {
        Ok(s) => s,
        Err(e) => {
            log::error!("{e}");
            //socket 5 通知失败信息
            let rep_code = 5;
            let socks5_response = build_response(&conn_dest, rep_code);
            if let Err(e1) = stream.write_all(&socks5_response).await {
                log::error!("write socks5_response failed: {e1}");
            }
            return;
        }
    };

    let mut is_ws_err = false;
    //把目标地址端口发给server,并检测server连接结果
    let rep = match check_server_conn(&mut ws_conn_pair, &conn_dest).await {
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
        let socks5_response = build_response(&conn_dest, rep);
        if let Err(e) = stream.write_all(&socks5_response).await {
            //回收连接
            if !is_ws_err {
                conn_manger.push_back_conn(ws_conn_pair).await;
            }
            log::error!("write socks5_response failed: {e}");
            return;
        }
    }
    //server连接remote失败
    if rep != 0 {
        //回收连接
        if !is_ws_err {
            conn_manger.push_back_conn(ws_conn_pair).await;
        }
        return;
    }
    let (tcp_reader, tcp_writer) = stream.split();
    let mut stream_reader = StreamReader::Socks5(tcp_reader);
    let mut stream_writer = StreamWriter::Socks5(tcp_writer);
    if let Err(proxy_error) = run_proxy_tcp_loop(
        &conn_manger,
        ws_conn_pair,
        &mut stream_reader,
        &mut stream_writer,
    )
    .await
    {
        log::error!("{proxy_error}");
    }
}
