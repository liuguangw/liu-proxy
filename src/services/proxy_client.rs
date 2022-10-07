mod check_server_conn;
mod handle_http_connection;
mod handle_socks5_connection;
mod http_server;
mod poll_message;
mod proxy_error;
mod proxy_handshake;
mod proxy_response_stream;
mod proxy_tcp;
mod run_proxy_tcp_loop;
mod send_message;
mod server_conn_manger;
mod socks5_server;
mod stream;

use crate::common::{ClientConfig, ClientError};
use crate::services;
use server_conn_manger::ServerConnManger;
use tokio::signal;

///运行客户端程序
pub async fn execute(config_file: &str) -> Result<(), ClientError> {
    let config: ClientConfig = services::load_config(config_file, "client")
        .await
        .map_err(|e| ClientError::Config(config_file.to_string(), e))?;
    //dbg!(&config);
    let conn_manger = ServerConnManger::try_init(&config)?;
    //连接服务端,测试连通性
    log::info!("check server status ...");
    let conn_pair = conn_manger
        .get_conn_pair()
        .await
        .map_err(ClientError::CheckConn)?;
    log::info!("server status ok");
    //把连接放回连接池
    conn_manger.push_back_conn(conn_pair).await;
    tokio::select! {
        output1= socks5_server::run_accept_loop(&conn_manger,&config) =>output1?,
        output2= http_server::run_accept_loop(&conn_manger,&config) =>output2?,
        output3 = signal::ctrl_c() =>{
            output3.map_err(ClientError::WaitSignal)?;
            log::info!(" - proxy server shutdown");
        },
    };
    Ok(())
}
