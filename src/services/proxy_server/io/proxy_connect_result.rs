use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
const MESSAGE_TYPE: u8 = 1;

///代表server连接远端的结果
pub enum ProxyConnectResult {
    ///成功
    Ok(TcpStream),
    ///io出错
    Err(std::io::Error),
    ///连接超时
    Timeout,
}
impl ProxyConnectResult {
    ///序列化
    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            Self::Ok(_) => vec![MESSAGE_TYPE, 0],
            Self::Err(e) => {
                let error_message = e.to_string();
                let mut raw_data = Vec::with_capacity(2 + error_message.len());
                raw_data.push(MESSAGE_TYPE);
                raw_data.push(1);
                raw_data.extend_from_slice(error_message.as_bytes());
                raw_data
            }
            Self::Timeout => vec![MESSAGE_TYPE, 2],
        }
    }
}

impl From<&ProxyConnectResult> for Message {
    fn from(action_result: &ProxyConnectResult) -> Self {
        let data = action_result.to_vec();
        Self::Binary(data)
    }
}
