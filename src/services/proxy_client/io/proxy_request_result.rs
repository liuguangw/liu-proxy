///代表远端请求的结果
pub enum ProxyRequestResult {
    ///成功
    Ok,
    ///io出错
    Err(String),
    ///远端主动关闭了连接
    Closed,
}

impl From<&[u8]> for ProxyRequestResult {
    fn from(data: &[u8]) -> Self {
        let ret_code = data[0];
        if ret_code == 0 {
            Self::Ok
        } else if ret_code == 1 {
            let error_message =
                std::str::from_utf8(&data[1..]).expect("parse error msg utf-8 failed");
            Self::Err(error_message.to_string())
        } else if ret_code == 2 {
            Self::Closed
        } else {
            panic!("invalid ProxyConnectResult ret_code: {ret_code}");
        }
    }
}
