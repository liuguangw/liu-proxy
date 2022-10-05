pub use super::client::{Connect, ProxyRequest};
use super::ParseMessageError;
use bytes::{Buf, Bytes};
//消息类型定义
const MESSAGE_TYPE_CONN: u8 = 0;
const MESSAGE_TYPE_DIS_CONN: u8 = 1;
const MESSAGE_TYPE_REQUEST: u8 = 2;

///客户端消息
pub enum ClientMessage {
    ///连接远端
    Conn(Connect),
    ///断开远端
    DisConn,
    ///写入请求
    Request(ProxyRequest),
}

impl From<Connect> for ClientMessage {
    fn from(item: Connect) -> Self {
        Self::Conn(item)
    }
}
impl From<ProxyRequest> for ClientMessage {
    fn from(item: ProxyRequest) -> Self {
        Self::Request(item)
    }
}

impl From<ClientMessage> for Bytes {
    ///序列化
    fn from(item: ClientMessage) -> Self {
        match item {
            ClientMessage::Conn(data) => {
                let data_bytes: Bytes = data.into();
                let mut buff = Bytes::from_static(&[MESSAGE_TYPE_CONN]).chain(data_bytes);
                buff.copy_to_bytes(buff.remaining())
            }
            ClientMessage::DisConn => Bytes::from_static(&[MESSAGE_TYPE_DIS_CONN]),
            ClientMessage::Request(data) => {
                let data_bytes: Bytes = data.into();
                let mut buff = Bytes::from_static(&[MESSAGE_TYPE_REQUEST]).chain(data_bytes);
                buff.copy_to_bytes(buff.remaining())
            }
        }
    }
}

impl TryFrom<Bytes> for ClientMessage {
    type Error = ParseMessageError;

    ///解析
    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ParseMessageError::Incomplete);
        }
        let msg_type = *value.first().unwrap();
        let message = if msg_type == MESSAGE_TYPE_CONN {
            let data_bytes = value.slice(1..);
            let sub_message = data_bytes.try_into()?;
            Self::Conn(sub_message)
        } else if msg_type == MESSAGE_TYPE_DIS_CONN {
            Self::DisConn
        } else if msg_type == MESSAGE_TYPE_REQUEST {
            let data_bytes = value.slice(1..);
            let sub_message = data_bytes.try_into()?;
            Self::Request(sub_message)
        } else {
            return Err(ParseMessageError::InvalidMsgType(msg_type));
        };
        Ok(message)
    }
}
