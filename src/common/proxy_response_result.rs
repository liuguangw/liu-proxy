use tokio_tungstenite::tungstenite::Message;

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
    //1[X]<data>
    pub fn message(&self) -> Message {
        let mut msg_data = Vec::with_capacity(2);
        msg_data.push(1);
        match self {
            Self::Ok(data) => {
                msg_data.push(0);
                let mut data_iter = data.iter();
                let next = || *data_iter.next().unwrap();
                msg_data.resize_with(2 + data.len(), next);
            }
            Self::Err(e) => {
                msg_data.push(1);
                let error_message = e.to_string();
                let mut data_iter = error_message.as_bytes().iter();
                let next = || *data_iter.next().unwrap();
                msg_data.resize_with(2 + error_message.len(), next);
            }
            Self::Closed => msg_data.push(2),
        };
        Message::binary(msg_data)
    }
}


impl ProxyResponseResult {
    pub fn to_vec(&self) -> Vec<u8> {
        todo!()
    }
}
