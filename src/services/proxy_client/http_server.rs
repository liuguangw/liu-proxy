use super::{
    handle_http_connection::handle_connection, proxy_response_stream::ProxyResponseStream,
    server_conn_manger::ServerConnManger,
};
use crate::common::{ClientConfig, ClientError};
use actix_web::{http, rt, web, App, HttpRequest, HttpResponse, HttpServer};
use tokio::sync::mpsc;

//http accept循环
pub async fn run_accept_loop(
    conn_manger: &ServerConnManger,
    config: &ClientConfig,
) -> Result<(), ClientError> {
    let addr = format!("{}:{}", &config.address, config.http_port);
    let conn_manger_clone = web::Data::new(conn_manger.clone());
    let server =
        HttpServer::new(move || App::new().configure(|cfg| configure_app(cfg, &conn_manger_clone)))
            .bind(&addr)
            .map_err(|e| ClientError::Bind(addr.to_string(), e))?;
    log::info!("http proxy listening on: {addr}");
    server.run().await.map_err(ClientError::HttpService)
}

#[actix_web::get("/")]
pub async fn hello() -> &'static str {
    "hello world"
}

pub async fn proxy_entry(
    req: HttpRequest,
    body: web::Payload,
    conn_manger: web::Data<ServerConnManger>,
) -> HttpResponse {
    let req_method = req.method();
    if req_method == http::Method::GET {
        return HttpResponse::NotFound().body("404 not found");
    } else if req_method == http::Method::CONNECT {
        let (tx, rx) = mpsc::channel::<Result<_, std::io::Error>>(5);
        let resp = ProxyResponseStream::new(rx).into();
        let target_addr = req.uri().to_string();
        rt::spawn(handle_connection(body, tx, target_addr, conn_manger));
        //response
        return resp;
    }
    HttpResponse::MethodNotAllowed().finish()
}

fn configure_app(cfg: &mut web::ServiceConfig, conn_manger: &web::Data<ServerConnManger>) {
    cfg.app_data(conn_manger.clone())
        .service(hello)
        .default_service(web::to(proxy_entry));
}
