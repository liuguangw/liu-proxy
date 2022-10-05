pub use super::server::{ConnectResult, ProxyResponseResult, RequestFail};
use super::ParseMessageError;
use bytes::{Buf, Bytes};
//消息类型定义
const MESSAGE_TYPE_CONN_RESULT: u8 = 0;
const MESSAGE_TYPE_RESPONSE_RESULT: u8 = 1;
const MESSAGE_TYPE_REQUEST_FAIL: u8 = 2;

///服务端消息
pub enum ServerMessage {
    ///连接远端的结果
    ConnResult(ConnectResult),
    ///响应结果
    ResponseResult(ProxyResponseResult),
    ///发送请求失败
    RequestFail(RequestFail),
}

impl From<ServerMessage> for Bytes {
    ///序列化
    fn from(item: ServerMessage) -> Self {
        match item {
            ServerMessage::ConnResult(data) => {
                let data_bytes: Bytes = data.into();
                let mut buff = Bytes::from_static(&[MESSAGE_TYPE_CONN_RESULT]).chain(data_bytes);
                buff.copy_to_bytes(buff.remaining())
            }
            ServerMessage::ResponseResult(data) => {
                let data_bytes: Bytes = data.into();
                let mut buff =
                    Bytes::from_static(&[MESSAGE_TYPE_RESPONSE_RESULT]).chain(data_bytes);
                buff.copy_to_bytes(buff.remaining())
            }
            ServerMessage::RequestFail(data) => {
                let data_bytes: Bytes = data.into();
                let mut buff = Bytes::from_static(&[MESSAGE_TYPE_REQUEST_FAIL]).chain(data_bytes);
                buff.copy_to_bytes(buff.remaining())
            }
        }
    }
}

impl TryFrom<Bytes> for ServerMessage {
    type Error = ParseMessageError;

    ///解析
    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ParseMessageError::Incomplete);
        }
        let msg_type = *value.first().unwrap();
        let message = if msg_type == MESSAGE_TYPE_CONN_RESULT {
            let data_bytes = value.slice(1..);
            let sub_message = data_bytes.try_into()?;
            Self::ConnResult(sub_message)
        } else if msg_type == MESSAGE_TYPE_RESPONSE_RESULT {
            let data_bytes = value.slice(1..);
            let sub_message = data_bytes.try_into()?;
            Self::ResponseResult(sub_message)
        } else if msg_type == MESSAGE_TYPE_REQUEST_FAIL {
            let data_bytes = value.slice(1..);
            let sub_message = data_bytes.try_into()?;
            Self::RequestFail(sub_message)
        } else {
            return Err(ParseMessageError::InvalidMsgType(msg_type));
        };
        Ok(message)
    }
}

impl From<ConnectResult> for ServerMessage {
    fn from(item: ConnectResult) -> Self {
        Self::ConnResult(item)
    }
}
impl From<ProxyResponseResult> for ServerMessage {
    fn from(item: ProxyResponseResult) -> Self {
        Self::ResponseResult(item)
    }
}
impl From<RequestFail> for ServerMessage {
    fn from(item: RequestFail) -> Self {
        Self::RequestFail(item)
    }
}
