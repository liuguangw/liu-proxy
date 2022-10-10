use std::{
    sync::Arc,
    time::{self, SystemTime},
};

use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use bytes::{Buf, Bytes};

use crate::common::ServerConfig;

pub struct CheckAuth {
    pub user: String,
}

#[async_trait]
impl<B> FromRequest<B> for CheckAuth
where
    B: Send, // required by `async_trait`
{
    type Rejection = http::StatusCode;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let upgrade_opt = req.headers().get(http::header::UPGRADE);
        let reject_resp = http::StatusCode::NOT_FOUND;
        //判断upgrade请求头
        match upgrade_opt {
            Some(s) if s == "websocket" => (),
            _ => return Err(reject_resp),
        };
        //获取Bearer token
        let bearer_token = match TypedHeader::<Authorization<Bearer>>::from_request(req).await {
            Ok(s) => s,
            Err(_) => return Err(reject_resp),
        };
        let bearer_token = bearer_token.token();
        //base64解码
        let decode_data = match base64::decode(bearer_token) {
            Ok(data) if data.len() > 28 => data,
            _ => return Err(reject_resp),
        };
        let buf = Bytes::from(decode_data);
        //计算用户名长度
        let user_length = buf.len() - 28;
        let user_bytes = buf.slice(..user_length);
        let username = match std::str::from_utf8(&user_bytes) {
            Ok(s) => s,
            Err(_) => return Err(reject_resp),
        };
        //后续部分
        let buf = buf.slice(user_length..);
        let mut ts_buf = buf.slice(..8);
        let ts = ts_buf.get_u64();
        //时间戳检测
        {
            let time_now = SystemTime::now();
            let current_ts = time_now.duration_since(time::UNIX_EPOCH).unwrap().as_secs();
            let diff_ts = if current_ts >= ts {
                current_ts - ts
            } else {
                ts - current_ts
            };
            //时间误差过大
            if diff_ts > 90 {
                return Err(reject_resp);
            }
        }
        let token = buf.slice(8..28);
        //
        let config = req.extensions().get::<Arc<ServerConfig>>().unwrap();
        for auth_user in &config.auth_users {
            if auth_user.user == username && auth_user.get_token(ts) == token {
                return Ok(Self {
                    user: username.to_string(),
                });
            }
        }
        Err(reject_resp)
    }
}
