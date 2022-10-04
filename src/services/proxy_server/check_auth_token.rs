use actix_web::{http::header, HttpRequest};

pub fn check_auth_token(req: &HttpRequest, auth_tokens: &[String]) -> bool {
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
    //获取token
    let token = &authorization_value[7..];
    //log::info!("check2#{token}");
    for t in auth_tokens {
        if t == token {
            return true;
        }
    }
    false
}
