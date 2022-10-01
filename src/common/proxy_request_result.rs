use tokio_tungstenite::tungstenite::Message;

///代表远端请求的结果
pub enum ProxyRequestResult {
    ///成功
    Ok,
    ///io出错
    Err(std::io::Error),
    ///远端主动关闭了连接
    Closed,
}

impl ProxyRequestResult {
    //0[X]<data>
    pub fn message(&self) -> Message {
        let mut msg_data = Vec::with_capacity(2);
        msg_data.push(0);
        match self {
            Self::Ok => {
                msg_data.push(0);
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

impl ProxyRequestResult {
    pub fn to_vec(&self) -> Vec<u8> {
        todo!()
    }
}
