use bytes::{BufMut, Bytes, BytesMut};
const MESSAGE_TYPE: u8 = 3;

///代表远端响应的结果
pub enum ProxyResponseResult {
    ///成功
    Ok(Bytes),
    ///io出错
    Err(std::io::Error),
    ///远端主动关闭了连接
    Closed,
}

impl ProxyResponseResult {
    ///序列化
    pub fn to_bytes(&self) -> Bytes {
        match self {
            Self::Ok(data) => {
                let mut raw_data = BytesMut::with_capacity(2 + data.len());
                raw_data.put_u8(MESSAGE_TYPE);
                raw_data.put_u8(0);
                raw_data.extend(data);
                raw_data.into()
            }
            Self::Err(e) => {
                let error_message = e.to_string();
                let mut raw_data = BytesMut::with_capacity(2 + error_message.len());
                raw_data.put_u8(MESSAGE_TYPE);
                raw_data.put_u8(1);
                raw_data.extend_from_slice(error_message.as_bytes());
                raw_data.into()
            }
            Self::Closed => Bytes::from_static(&[MESSAGE_TYPE, 2]),
        }
    }
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }
}
