///代表server连接远端的结果
pub enum ProxyConnectResult {
    ///成功
    Ok,
    ///io出错
    Err(std::io::Error),
    ///连接超时
    Timeout,
}
impl ProxyConnectResult {
    pub fn to_vec(&self) -> Vec<u8> {
        todo!()
    }
}
