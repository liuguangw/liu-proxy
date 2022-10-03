mod check_auth_token;
mod handle_connection;
mod io;
mod proxy_error;
mod proxy_tcp;
mod run_proxy_tcp_loop;
mod tls;
mod wait_conn_remote;

use crate::common::ServerConfig;
use crate::services;
pub use handle_connection::handle_connection as handle_connection_fn;
use tokio::net::TcpListener;
use tokio::signal;

///运行服务端程序
pub async fn execute(config_file: &str) -> Result<(), String> {
    let config: ServerConfig = match services::load_config(config_file, "server").await {
        Ok(s) => s,
        Err(e) => return Err(format!("load {config_file} failed: {e}")),
    };
    //dbg!(&config);
    let addr = format!("{}:{}", &config.address, config.port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(s) => s,
        Err(e) => return Err(format!("bind {addr} failed: {e}")),
    };
    println!("Listening on: {addr}");
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

async fn run_accept_loop(listener: TcpListener, config: ServerConfig) {
    let tls_acceptor = if config.use_ssl {
        match tls::load_tls_acceptor(&config).await {
            Ok(s) => Some(s),
            Err(e) => {
                eprintln!("init tls failed: {}", e);
                return;
            }
        }
    } else {
        None
    };
    loop {
        let (stream, addr) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                println!("accept tcp failed: {}", e);
                continue;
            }
        };
        tokio::spawn(handle_connection_fn(
            stream,
            addr,
            config.to_owned(),
            tls_acceptor.to_owned(),
        ));
    }
}
