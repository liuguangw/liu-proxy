use tokio_tungstenite::tungstenite::Message;
const MESSAGE_TYPE: u8 = 2;

///代表远端请求的结果
pub enum ProxyRequestResult {
    ///成功
    Ok,
    ///io出错
    Err(std::io::Error),
    ///远端主动关闭了连接
    Closed,
}

impl ProxyRequestResult {
    ///序列化
    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            Self::Ok => vec![MESSAGE_TYPE, 0],
            Self::Err(e) => {
                let error_message = e.to_string();
                let mut raw_data = Vec::with_capacity(2 + error_message.len());
                raw_data.push(MESSAGE_TYPE);
                raw_data.push(1);
                raw_data.extend_from_slice(error_message.as_bytes());
                raw_data
            }
            Self::Closed => vec![MESSAGE_TYPE, 2],
        }
    }
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }
}
impl From<&ProxyRequestResult> for Message {
    fn from(action_result: &ProxyRequestResult) -> Self {
        let data = action_result.to_vec();
        Self::Binary(data)
    }
}
