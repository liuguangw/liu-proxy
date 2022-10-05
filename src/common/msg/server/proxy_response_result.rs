use super::super::ParseMessageError;
use bytes::{Buf, Bytes};
//状态定义
const STATUS_OK: u8 = 0;
const STATUS_ERR: u8 = 1;
const STATUS_CLOSED: u8 = 2;

///response
pub enum ProxyResponseResult {
    ///成功
    Ok(Bytes),
    ///io出错
    Err(String),
    ///远端主动关闭了连接
    Closed,
}

impl From<ProxyResponseResult> for Bytes {
    ///序列化
    fn from(item: ProxyResponseResult) -> Self {
        match item {
            ProxyResponseResult::Ok(data) => {
                let mut buff = Bytes::from_static(&[STATUS_OK]).chain(data);
                buff.copy_to_bytes(buff.remaining())
            }
            ProxyResponseResult::Err(message_str) => {
                let message_buff = Bytes::from(message_str);
                let mut buff = Bytes::from_static(&[STATUS_ERR]).chain(message_buff);
                buff.copy_to_bytes(buff.remaining())
            }
            ProxyResponseResult::Closed => Bytes::from_static(&[STATUS_CLOSED]),
        }
    }
}

impl TryFrom<Bytes> for ProxyResponseResult {
    type Error = ParseMessageError;

    ///解析
    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ParseMessageError::Incomplete);
        }
        let response_status = *value.first().unwrap();
        let payload_result = if response_status == STATUS_OK {
            let data = value.slice(1..);
            Self::Ok(data)
        } else if response_status == STATUS_ERR {
            let message_str = value.slice(1..);
            let message_str = std::str::from_utf8(&message_str)?;
            Self::Err(message_str.to_string())
        } else if response_status == STATUS_CLOSED {
            Self::Closed
        } else {
            return Err(ParseMessageError::InvalidResponseStatus(response_status));
        };
        Ok(payload_result)
    }
}
