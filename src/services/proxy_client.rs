mod handle_connection;
mod proxy_handshake;
mod check_auth_token;
mod run_proxy_tcp_loop;
mod proxy_tcp;
mod check_server_conn;
mod proxy_error;

use std::io::Result as IoResult;
use tokio::net::TcpListener;
use tokio::signal;
pub use handle_connection::handle_connection as handle_connection_fn;

pub async fn execute(listen_address: (&str, u16), server_address: (&str, u16)) -> IoResult<()> {
    let listener = TcpListener::bind(listen_address).await?;
    println!(
        "Socket5 Listening on: {}:{}",
        listen_address.0, listen_address.1
    );
    let server_address = format!("{}:{}",server_address.0,server_address.1);
    tokio::select! {
        _ = run_accept_loop(listener, server_address) =>(),
        output2 = signal::ctrl_c() =>{
            output2?;
            println!(" - client shutdown");
        },
    };
    Ok(())
}

async fn run_accept_loop(listener: TcpListener, server_address: String) {
    loop {
        let (stream, addr) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                println!("accept failed: {}", e);
                continue;
            }
        };
        tokio::spawn(handle_connection_fn(stream, addr,server_address.clone()));
    }
}
