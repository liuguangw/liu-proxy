use std::time::{self, SystemTime};

use actix_web::{http::header, HttpRequest};
use bytes::{Buf, Bytes};

use crate::common::AuthUser;

pub fn check_auth_user(req: &HttpRequest, auth_users: &[AuthUser]) -> bool {
    //初步检测
    match req.headers().get(header::UPGRADE) {
        Some(s) if s == "websocket" => (),
        _ => return false,
    };
    let authorization_value = match req.headers().get(header::AUTHORIZATION) {
        Some(s) => match s.to_str() {
            Ok(s) => s,
            Err(_) => return false,
        },
        None => return false,
    };
    //log::info!("check1#{authorization_value}");
    if !authorization_value.starts_with("Bearer ") {
        return false;
    }
    //获取encoded_buf
    let encoded_buf = &authorization_value[7..];
    let decode_data = match base64::decode(encoded_buf) {
        Ok(data) if data.len() > 28 => data,
        _ => return false,
    };
    let buf = Bytes::from(decode_data);
    //计算用户名长度
    let user_length = buf.len() - 28;
    let user_bytes = buf.slice(..user_length);
    let user = match std::str::from_utf8(&user_bytes) {
        Ok(s) => s,
        Err(_) => return false,
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
            return false;
        }
    }
    let token = buf.slice(8..28);
    for auth_user in auth_users {
        if auth_user.user == user {
            return auth_user.get_token(ts) == token;
        }
    }
    false
}
