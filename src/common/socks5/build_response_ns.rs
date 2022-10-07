use super::ConnDest;

///构造socks5响应码结果
pub fn build_response(conn_dest: &ConnDest, code: u8) -> Vec<u8> {
    let addr_raw_data = conn_dest.to_raw_data();
    let mut socks5_response = Vec::with_capacity(3 + addr_raw_data.len());
    socks5_response.push(super::VERSION);
    socks5_response.push(code);
    socks5_response.push(0);
    socks5_response.extend_from_slice(&addr_raw_data);
    socks5_response
}
