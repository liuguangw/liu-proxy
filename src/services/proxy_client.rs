mod check_auth_token;
mod check_server_conn;
mod handle_connection;
mod io;
mod proxy_error;
mod proxy_handshake;
mod proxy_tcp;
mod run_proxy_tcp_loop;

pub use handle_connection::handle_connection as handle_connection_fn;
use std::io::Result as IoResult;
use tokio::net::TcpListener;
use tokio::signal;

pub async fn execute(
    listen_address: (&str, u16),
    server_url: &str,
    server_host: &Option<String>,
) -> IoResult<()> {
    let listener = TcpListener::bind(listen_address).await?;
    println!(
        "Socket5 Listening on: {}:{}",
        listen_address.0, listen_address.1
    );
    //let server_address = server_url.to_string();
    tokio::select! {
        _ = run_accept_loop(listener, server_url, server_host) =>(),
        output2 = signal::ctrl_c() =>{
            output2?;
            println!(" - client shutdown");
        },
    };
    Ok(())
}

async fn run_accept_loop(listener: TcpListener, server_url: &str, server_host: &Option<String>) {
    loop {
        let (stream, addr) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                println!("accept failed: {}", e);
                continue;
            }
        };
        tokio::spawn(handle_connection_fn(
            stream,
            addr,
            server_url.to_string(),
            server_host.to_owned(),
        ));
    }
}
