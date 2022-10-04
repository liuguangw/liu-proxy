use super::handle_connection::handle_connection;
use crate::common::ServerConfig;
use actix_web::{
    http::{header, Method},
    rt, web, Either, HttpRequest, HttpResponse, Responder,
};

///处理websocket连接
pub async fn ws_handler(
    req: HttpRequest,
    body: web::Payload,
    config: web::Data<ServerConfig>,
) -> Result<Either<HttpResponse, impl Responder>, actix_web::Error> {
    //初步检测
    match req.headers().get(header::UPGRADE) {
        Some(s) if s == "websocket" => (),
        _ => {
            //404
            let err404_response = super::error_404_handler(Method::GET).await?;
            return Ok(Either::Right(err404_response));
        }
    };
    //执行握手
    let (response, session, msg_stream) = actix_ws::handle(&req, body)?;
    rt::spawn(handle_connection(session, msg_stream, config));
    Ok(Either::Left(response))
}
