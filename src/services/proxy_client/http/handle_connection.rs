use super::{
    super::{run_proxy_tcp_loop::run_proxy_tcp_loop, server_conn_manger::ServerConnManger},
    build_request,
    parse_request::{parse_conn_dest, parse_request_body_size},
    run_proxy_request_loop::run_proxy_request_loop,
    write_handshake_response::write_handshake_response,
};
use crate::{
    common::RouteConfigCom,
    services::{
        proxy_client::connection::{ConnectionError, RemoteConnection},
        read_raw_data,
    },
};
use bytes::{BufMut, BytesMut};
use httparse::{Request, Status};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpStream;

///处理http连接
pub async fn handle_connection(
    mut stream: TcpStream,
    _addr: SocketAddr,
    conn_manger: ServerConnManger,
    route_config: Arc<RouteConfigCom>,
    first_byte: u8,
) {
    let mut buf = BytesMut::new();
    buf.put_u8(first_byte);
    loop {
        //读取数据
        let raw_data = match read_raw_data::read_raw(&mut stream).await {
            Ok(s) => s,
            Err(e) => {
                log::error!("read request raw data failed: {e}");
                return;
            }
        };
        buf.put_slice(&raw_data);
        //解析
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);
        let offset_status = match req.parse(&buf) {
            Ok(s) => s,
            Err(e) => {
                log::error!("parse http request failed: {e}");
                return;
            }
        };
        //解析到完整的header头
        if let Status::Complete(body_offset) = offset_status {
            handle_request(stream, conn_manger, route_config, &req, &buf, body_offset).await;
            break;
        }
    }
}

async fn handle_request(
    mut stream: TcpStream,
    conn_manger: ServerConnManger,
    route_config: Arc<RouteConfigCom>,
    req: &Request<'_, '_>,
    buf: &[u8],
    body_offset: usize,
) {
    let req_method = req.method.unwrap();
    let req_path = req.path.unwrap();
    let (is_connect, conn_dest) = if req_method == http::Method::CONNECT {
        (true, req_path.to_string())
    } else {
        //log::info!("proxy {req_method} {req_path}");
        match parse_conn_dest(req_path) {
            Ok(s) => (false, s),
            Err(e) => {
                log::error!("get conn_dest failed: {e}");
                return;
            }
        }
    };
    let http_version = if req.version.unwrap() == 1 {
        "1.1"
    } else {
        "1.0"
    };
    let remote_conn = match RemoteConnection::connect(&conn_dest, &conn_manger, &route_config).await
    {
        Ok(s) => s,
        Err(e) => {
            if !matches!(e, ConnectionError::RouteBlocked) {
                log::error!("{e}");
            }
            //http 通知失败信息
            if let Err(e1) =
                write_handshake_response(&mut stream, http_version, req_path, false).await
            {
                log::error!("write http_response failed: {e1}");
            }
            return;
        }
    };
    //CONNECT请求: 写入http_response
    if is_connect {
        if let Err(e) = write_handshake_response(&mut stream, http_version, req_path, true).await {
            //回收连接
            remote_conn.push_back_conn(&conn_manger).await;
            log::error!("write http_response failed: {e}");
            return;
        }
    }
    let proxy_result = if is_connect {
        run_proxy_tcp_loop(&conn_manger, remote_conn, stream).await
    } else {
        let first_request_data = build_request::build_request_data(req, buf, body_offset);
        let body_total_size = match parse_request_body_size(req.headers) {
            Ok(s) => s,
            Err(e) => {
                log::error!("parse body size failed: {e}");
                //回收连接
                remote_conn.push_back_conn(&conn_manger).await;
                return;
            }
        };
        let remain_data_size = if body_total_size > 0 {
            body_total_size - (buf.len() - body_offset)
        } else {
            0
        };
        run_proxy_request_loop(
            &conn_manger,
            remote_conn,
            stream,
            first_request_data,
            remain_data_size,
        )
        .await
    };
    //proxy
    if let Err(proxy_error) = proxy_result {
        log::error!("{proxy_error}");
    }
}
