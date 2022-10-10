mod check_auth_user;
mod handle_connection;
mod poll_message;
mod proxy_error;
mod proxy_tcp;
mod run_proxy_tcp_loop;
mod send_message;
mod tls;
mod ws_handler_ns;

use crate::common::{ServerConfig, ServerError};
use crate::services;
use actix_files::{Files, NamedFile};
use actix_web::http::{Method, StatusCode};
use actix_web::{web, App, Either, HttpResponse, HttpServer, Responder};

use self::ws_handler_ns::ws_handler;

///运行服务端程序
pub async fn execute(config_file: &str) -> Result<(), ServerError> {
    //加载配置
    let config: ServerConfig = services::load_config(config_file, "server")
        .await
        .map_err(|e| ServerError::Config(config_file.to_string(), e))?;
    let config = web::Data::new(config);
    //address
    let listen_address = format!("{}:{}", &config.address, config.port);
    log::info!(
        "Server listen {listen_address} (ssl = {:?})",
        config.use_ssl
    );
    //build server
    let server = {
        let config_clone = config.clone();
        HttpServer::new(move || App::new().configure(|cfg| configure_app(cfg, &config_clone)))
    };
    //判断是否开启ssl
    let server = if config.use_ssl {
        //cert路径
        let ssl_cert_path = match &config.ssl_cert_path {
            Some(s) if !s.is_empty() => s.as_str(),
            _ => return Err(ServerError::ConfigSSlCertNone),
        };
        //key路径
        let ssl_key_path = match &config.ssl_key_path {
            Some(s) if !s.is_empty() => s.as_str(),
            _ => return Err(ServerError::ConfigSSlKeyNone),
        };
        let tls_config = tls::load_tls_config(ssl_cert_path, ssl_key_path).await?;
        server.bind_rustls(&listen_address, tls_config)
    } else {
        server.bind(&listen_address)
    }
    .map_err(|e| ServerError::Bind(listen_address.to_string(), e))?;
    //worker数量配置
    let server = match &config.worker_count {
        Some(s) => server.workers(*s),
        None => server,
    };
    //run
    server.run().await.map_err(ServerError::HttpService)?;
    log::info!("proxy server shutdown");
    Ok(())
}

///用于显示404错误页面
async fn error_404_handler(req_method: Method) -> Result<impl Responder, actix_web::Error> {
    match req_method {
        Method::GET => {
            let file = NamedFile::open_async("./web/404.html").await?;
            let resp = file.customize().with_status(StatusCode::NOT_FOUND);
            Ok(Either::Left(resp))
        }
        _ => Ok(Either::Right(HttpResponse::MethodNotAllowed().finish())),
    }
}

fn configure_app(cfg: &mut web::ServiceConfig, config: &web::Data<ServerConfig>) {
    cfg.route(&config.path, web::get().to(ws_handler))
        //挂载静态文件夹
        .service(
            Files::new("/", "./web/public")
                .prefer_utf8(true)
                .redirect_to_slash_directory()
                .index_file("index.html"),
        )
        .app_data(config.clone())
        .default_service(web::to(error_404_handler));
}
