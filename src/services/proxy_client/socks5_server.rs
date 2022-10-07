use super::handle_socks5_connection::handle_connection;
use super::server_conn_manger::ServerConnManger;
use crate::common::{ClientConfig, ClientError};
use actix_web::rt;
use tokio::net::TcpListener;

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
        rt::spawn(handle_connection(stream, addr, conn_manger.to_owned()));
    }
}
