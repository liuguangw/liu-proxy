mod check_server_conn;
mod handle_connection;
mod poll_message;
mod proxy_error;
mod proxy_handshake;
mod proxy_tcp;
mod run_proxy_tcp_loop;
mod send_message;
mod server_conn_manger;

use crate::common::{ClientConfig, ClientError};
use crate::services;
use handle_connection::handle_connection as handle_connection_fn;
use server_conn_manger::ServerConnManger;
use tokio::net::TcpListener;
use tokio::signal;

///运行客户端程序
pub async fn execute(config_file: &str) -> Result<(), ClientError> {
    let config: ClientConfig = services::load_config(config_file, "client")
        .await
        .map_err(|e| ClientError::Config(config_file.to_string(), e))?;
    //dbg!(&config);
    let addr = format!("{}:{}", &config.address, config.port);
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| ClientError::Bind(addr.to_string(), e))?;
    log::info!("Socket5 Listening on: {addr}");
    //let server_address = server_url.to_string();
    tokio::select! {
        _ = run_accept_loop(listener, config) =>(),
        output2 = signal::ctrl_c() =>{
            output2.map_err(ClientError::WaitSignal)?;
            log::info!(" - proxy server shutdown");
        },
    };
    Ok(())
}

async fn run_accept_loop(listener: TcpListener, config: ClientConfig) {
    let conn_manger = match ServerConnManger::try_init(&config) {
        Ok(s) => s,
        Err(e) => {
            log::error!("{e}");
            return;
        }
    };
    //连接服务端,测试连通性
    log::info!("check server status ...");
    match conn_manger.get_conn_pair().await {
        Ok(conn_pair) => {
            log::info!("server status ok");
            conn_manger.push_back_conn(conn_pair).await;
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
        tokio::spawn(handle_connection_fn(stream, addr, conn_manger.to_owned()));
    }
}
