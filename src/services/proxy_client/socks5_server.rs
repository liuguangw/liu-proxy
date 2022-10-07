use super::handle_socks5_connection::handle_connection;
use super::server_conn_manger::ServerConnManger;
use crate::common::{ClientConfig, ClientError};
use tokio::net::TcpListener;
use tokio::task;

//socks5 accept循环
pub async fn run_accept_loop(
    conn_manger: &ServerConnManger,
    config: &ClientConfig,
) -> Result<(), ClientError> {
    let addr = format!("{}:{}", &config.address, config.socks5_port);
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| ClientError::Bind(addr.to_string(), e))?;
    log::info!("socks5 proxy listening on: {addr}");
    loop {
        let (stream, addr) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                println!("accept failed: {}", e);
                continue;
            }
        };
        //spawn_local
        let local = task::LocalSet::new();
        local
            .run_until(async {
                task::spawn_local(handle_connection(stream, addr, conn_manger.to_owned()))
                    .await
                    .unwrap();
            })
            .await;
    }
}
