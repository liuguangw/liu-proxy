use super::super::ParseMessageError;
use bytes::{Buf, Bytes};
//状态定义
const STATUS_OK: u8 = 0;
const STATUS_ERR: u8 = 1;
const STATUS_TIMEOUT: u8 = 2;

///连接remote的结果
pub enum ConnectResult {
    ///成功
    Ok,
    ///io出错
    Err(String),
    ///连接超时
    Timeout,
}

impl From<ConnectResult> for Bytes {
    ///序列化
    fn from(item: ConnectResult) -> Self {
        match item {
            ConnectResult::Ok => Bytes::from_static(&[STATUS_OK]),
            ConnectResult::Err(message_str) => {
                let message_buff = Bytes::from(message_str);
                let mut buff = Bytes::from_static(&[STATUS_ERR]).chain(message_buff);
                buff.copy_to_bytes(buff.remaining())
            }
            ConnectResult::Timeout => Bytes::from_static(&[STATUS_TIMEOUT]),
        }
    }
}

impl TryFrom<Bytes> for ConnectResult {
    type Error = ParseMessageError;

    ///解析
    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ParseMessageError::Incomplete);
        }
        let conn_status = *value.first().unwrap();
        let item = if conn_status == STATUS_OK {
            Self::Ok
        } else if conn_status == STATUS_ERR {
            let message_str = value.slice(1..);
            let message_str = std::str::from_utf8(&message_str)?;
            Self::Err(message_str.to_string())
        } else if conn_status == STATUS_TIMEOUT {
            Self::Timeout
        } else {
            return Err(ParseMessageError::InvalidConnStatus(conn_status));
        };
        Ok(item)
    }
}
