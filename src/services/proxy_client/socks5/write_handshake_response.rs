use crate::common::socks5::{ConnDest, VERSION};
use std::io::Result as IoResult;
use tokio::{io::AsyncWriteExt, net::TcpStream};

pub async fn write_handshake_response(
    stream: &mut TcpStream,
    conn_dest: &ConnDest,
    is_ok: bool,
) -> IoResult<()> {
    let rep = if is_ok { 0 } else { 5 };
    let addr_raw_data = conn_dest.to_raw_data();
    let mut socks5_response = Vec::with_capacity(3 + addr_raw_data.len());
    socks5_response.push(VERSION);
    socks5_response.push(rep);
    socks5_response.push(0);
    socks5_response.extend_from_slice(&addr_raw_data);
    stream.write_all(&socks5_response).await
}
