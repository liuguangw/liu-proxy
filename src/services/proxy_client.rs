mod auth_handshake_ns;
mod check_server_conn;
mod handle_connection;
mod io;
mod poll_message;
mod proxy_error;
mod proxy_handshake;
mod proxy_tcp;
mod run_proxy_tcp_loop;

use crate::common::{ClientConfig, WebsocketRequest};
use crate::services;
use auth_handshake_ns::auth_handshake;
use handle_connection::handle_connection as handle_connection_fn;
use tokio::net::TcpListener;
use tokio::signal;

///运行客户端程序
pub async fn execute(config_file: &str) -> Result<(), String> {
    let config: ClientConfig = match services::load_config(config_file, "client").await {
        Ok(s) => s,
        Err(e) => return Err(format!("load {config_file} failed: {e}")),
    };
    //dbg!(&config);
    let addr = format!("{}:{}", &config.address, config.port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(s) => s,
        Err(e) => return Err(format!("bind {addr} failed: {e}")),
    };
    log::info!("Socket5 Listening on: {addr}");
    //let server_address = server_url.to_string();
    tokio::select! {
        _ = run_accept_loop(listener, config) =>(),
        output2 = signal::ctrl_c() =>{
            if let Err(e) = output2{
                return Err(format!("wait signal failed: {e}"));
            }
            log::info!(" - proxy server shutdown");
        },
    };
    Ok(())
}

async fn run_accept_loop(listener: TcpListener, config: ClientConfig) {
    let ws_request = match WebsocketRequest::try_from(&config) {
        Ok(s) => s,
        Err(e) => {
            log::error!("{e}");
            return;
        }
    };
    //连接服务端,测试连通性
    log::info!("check server status ...");
    match auth_handshake(&ws_request).await {
        Ok(s) => {
            log::info!("server status ok");
            //关闭测试连接
            let mut ws_stream = s.0;
            if let Err(e) = ws_stream.close(None).await {
                log::warn!("close test websocket stream failed: {e}");
            }
        }
        Err(e) => {
            log::error!("{e}");
            return;
        }
    };
    loop {
        let (stream, addr) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                println!("accept failed: {}", e);
                continue;
            }
        };
        tokio::spawn(handle_connection_fn(stream, addr, ws_request.clone()));
    }
}
