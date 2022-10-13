use super::super::ParseMessageError;
use bytes::Bytes;

///request
#[derive(Debug)]
pub struct ProxyRequest(pub Bytes);

impl From<ProxyRequest> for Bytes {
    ///序列化
    fn from(item: ProxyRequest) -> Self {
        item.0
    }
}

impl TryFrom<Bytes> for ProxyRequest {
    type Error = ParseMessageError;

    ///解析
    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ParseMessageError::Incomplete);
        }
        Ok(Self(value))
    }
}
