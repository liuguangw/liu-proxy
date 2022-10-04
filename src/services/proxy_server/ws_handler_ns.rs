use super::check_auth_token::check_auth_token;
use super::handle_connection::handle_connection;
use crate::common::ServerConfig;
use actix_web::{http::Method, rt, web, Either, HttpRequest, HttpResponse, Responder};

///处理websocket连接
pub async fn ws_handler(
    req: HttpRequest,
    body: web::Payload,
    config: web::Data<ServerConfig>,
) -> Result<Either<HttpResponse, impl Responder>, actix_web::Error> {
    //auth
    if !check_auth_token(&req, &config.auth_tokens) {
        //404
        let err404_response = super::error_404_handler(Method::GET).await?;
        return Ok(Either::Right(err404_response));
    }
    //执行握手
    let (response, session, msg_stream) = actix_ws::handle(&req, body)?;
    rt::spawn(handle_connection(session, msg_stream));
    Ok(Either::Left(response))
}
