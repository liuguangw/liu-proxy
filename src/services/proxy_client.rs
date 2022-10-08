mod check_server_conn;
mod handle_connection;
mod http;
mod poll_message;
mod proxy_error;
mod proxy_tcp;
mod run_proxy_tcp_loop;
mod send_message;
mod server_conn_manger;
mod socks5;

use crate::common::{ClientConfig, ClientError};
use crate::services;
use server_conn_manger::ServerConnManger;
use tokio::net::TcpListener;
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
        output1= run_accept_loop(conn_manger,config) =>output1?,
        output2 = signal::ctrl_c() =>{
            output2.map_err(ClientError::WaitSignal)?;
            log::info!(" - proxy server shutdown");
        },
    };
    Ok(())
}

//accept tcp 连接循环
async fn run_accept_loop(
    conn_manger: ServerConnManger,
    config: ClientConfig,
) -> Result<(), ClientError> {
    let addr = format!("{}:{}", &config.address, config.port);
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| ClientError::Bind(addr.to_string(), e))?;
    log::info!("proxy listening on: {addr}");
    loop {
        let (stream, addr) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                println!("accept failed: {}", e);
                continue;
            }
        };
        //spawn
        tokio::spawn(handle_connection::handle_connection(
            stream,
            addr,
            conn_manger.to_owned(),
        ));
    }
}
