use bytes::{BufMut, Bytes, BytesMut};
use tokio::net::TcpStream;
const MESSAGE_TYPE: u8 = 1;

///代表server连接远端的结果
pub enum ProxyConnectResult {
    ///成功
    Ok(TcpStream),
    ///io出错
    Err(std::io::Error),
    ///连接超时
    Timeout,
}
impl ProxyConnectResult {
    ///序列化
    pub fn to_bytes(&self) -> Bytes {
        match self {
            Self::Ok(_) => Bytes::from_static(&[MESSAGE_TYPE, 0]),
            Self::Err(e) => {
                let error_message = e.to_string();
                let mut raw_data = BytesMut::with_capacity(2 + error_message.len());
                raw_data.put_u8(MESSAGE_TYPE);
                raw_data.put_u8(1);
                raw_data.extend_from_slice(error_message.as_bytes());
                raw_data.into()
            }
            Self::Timeout => Bytes::from_static(&[MESSAGE_TYPE, 2]),
        }
    }
}
