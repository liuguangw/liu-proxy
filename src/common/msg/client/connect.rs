use super::super::ParseMessageError;
use bytes::Bytes;

///连接remote
pub struct Connect(pub String);

impl From<Connect> for Bytes {
    ///序列化
    fn from(item: Connect) -> Self {
        Bytes::from(item.0)
    }
}

impl TryFrom<Bytes> for Connect {
    type Error = ParseMessageError;

    ///解析
    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ParseMessageError::Incomplete);
        }
        let value_str = std::str::from_utf8(&value)?;
        Ok(Self(value_str.to_string()))
    }
}
