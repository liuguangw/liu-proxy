use super::ConnDest;

///构造socket5响应码结果
pub fn build_response(conn_dest: &ConnDest, code: u8) -> Vec<u8> {
    let addr_raw_data = conn_dest.to_raw_data();
    let mut socket5_response = Vec::with_capacity(3 + addr_raw_data.len());
    socket5_response.push(super::VERSION);
    socket5_response.push(code);
    socket5_response.push(0);
    socket5_response.extend_from_slice(&addr_raw_data);
    socket5_response
}
