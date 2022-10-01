use tokio_tungstenite::tungstenite::Message;

use super::{ProxyConnectResult, ProxyRequestResult, ProxyResponseResult};

///代表server端对remote资源的IO结果
///
/// server端得到结果后,会把此结果转发给客户端
pub enum ProxyIoResult {
    ///连接
    Connect(ProxyConnectResult),
    ///发送请求
    Request(ProxyRequestResult),
    ///读取响应
    Response(ProxyResponseResult),
}

impl ProxyIoResult {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut data = vec![];
        let (result_type, result_data) = match self {
            ProxyIoResult::Connect(s) => (1, s.to_vec()),
            ProxyIoResult::Request(s) => (2, s.to_vec()),
            ProxyIoResult::Response(s) => (3, s.to_vec()),
        };
        data.push(result_type);
        data.extend_from_slice(&result_data);
        data
    }
}

impl From<&ProxyIoResult> for Message {
    fn from(io_result: &ProxyIoResult) -> Self {
        let data = io_result.to_vec();
        Self::Binary(data)
    }
}
