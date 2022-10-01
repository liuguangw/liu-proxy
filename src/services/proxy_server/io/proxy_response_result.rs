use tokio_tungstenite::tungstenite::Message;
const MESSAGE_TYPE: u8 = 3;

///代表远端响应的结果
pub enum ProxyResponseResult {
    ///成功
    Ok(Vec<u8>),
    ///io出错
    Err(std::io::Error),
    ///远端主动关闭了连接
    Closed,
}

impl ProxyResponseResult {
    ///序列化
    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            Self::Ok(data) => {
                let mut raw_data = Vec::with_capacity(2 + data.len());
                raw_data.push(MESSAGE_TYPE);
                raw_data.push(0);
                raw_data.extend_from_slice(data);
                raw_data
            }
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
        matches!(self, Self::Ok(_))
    }
}

impl From<&ProxyResponseResult> for Message {
    fn from(action_result: &ProxyResponseResult) -> Self {
        let data = action_result.to_vec();
        Self::Binary(data)
    }
}
