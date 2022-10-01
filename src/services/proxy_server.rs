mod handle_connection;
mod check_auth_token;
mod run_proxy_tcp_loop;
mod proxy_tcp;
mod proxy_error;
mod wait_conn_remote;

use std::io::Result as IoResult;
use tokio::net::TcpListener;
use tokio::signal;
pub use handle_connection::handle_connection as handle_connection_fn;

pub async fn execute(address: &str, port: u16) -> IoResult<()> {
    let listener = TcpListener::bind((address, port)).await?;
    println!("Listening on: {address}:{port}");
    tokio::select! {
        _ = run_accept_loop(listener) =>(),
        output2 = signal::ctrl_c() =>{
            output2?;
            println!(" - proxy server shutdown");
        },
    };
    Ok(())
}

async fn run_accept_loop(listener: TcpListener) {
    loop {
        let (stream, addr) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                println!("accept failed: {}", e);
                continue;
            }
        };
        tokio::spawn(handle_connection_fn(stream, addr));
    }
}
