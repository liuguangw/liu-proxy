mod auth_handshake_ns;
mod check_auth_token;
mod check_server_conn;
mod handle_connection;
mod io;
mod proxy_error;
mod proxy_handshake;
mod proxy_tcp;
mod run_proxy_tcp_loop;
use crate::common::ClientConfig;
use crate::services;
use auth_handshake_ns::auth_handshake;
pub use handle_connection::handle_connection as handle_connection_fn;
use std::time::Duration;
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
    println!("Socket5 Listening on: {addr}");
    //let server_address = server_url.to_string();
    tokio::select! {
        _ = run_accept_loop(listener, config) =>(),
        output2 = signal::ctrl_c() =>{
            if let Err(e) = output2{
                return Err(format!("wait signal failed: {e}"));
            }
            println!(" - proxy server shutdown");
        },
    };
    Ok(())
}

async fn run_accept_loop(listener: TcpListener, config: ClientConfig) {
    //连接服务端测试连通性
    match auth_handshake(&config, Duration::from_secs(8)).await {
        Ok(s) => {
            let mut ws_stream = s.0;
            if let Err(e) = ws_stream.close(None).await {
                eprintln!("close test websocket stream failed: {e}");
            }
        }
        Err(e) => {
            eprintln!("{e}");
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
        tokio::spawn(handle_connection_fn(stream, addr, config.clone()));
    }
}
