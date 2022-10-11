mod check_auth;
mod handle_connection;
mod poll_message;
mod proxy_error;
mod proxy_tcp;
mod run_proxy_tcp_loop;
mod send_message;
mod ws_handler_ns;

use crate::common::{ServerConfig, ServerError};
use axum::response::IntoResponse;
use axum::{routing, Extension, Router};
use axum_server::tls_rustls::RustlsConfig;
use http::StatusCode;
use std::io::Error as IoError;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use tokio::signal;
use tower_http::services::{ServeDir, ServeFile};

///运行服务端程序
pub async fn execute(config: ServerConfig) -> Result<(), ServerError> {
    let config = Arc::new(config);
    let mut addrs_iter = (config.address.as_str(), config.port)
        .to_socket_addrs()
        .map_err(ServerError::ParseAddress)?;
    //取第一个地址
    let mut listen_address = addrs_iter.next().unwrap();
    //ipv4优先绑定
    if !listen_address.is_ipv4() {
        for addr in addrs_iter {
            if addr.is_ipv4() {
                listen_address = addr;
                break;
            }
        }
    }
    //判断是否开启ssl
    if !config.use_ssl {
        let app = build_app(config);
        run_http(app, &listen_address).await?;
    } else {
        let app = build_app(config.clone());
        let cert_path = match &config.ssl_cert_path {
            Some(s) => s.as_str(),
            None => return Err(ServerError::ConfigSSlCertNone),
        };
        let key_path = match &config.ssl_key_path {
            Some(s) => s.as_str(),
            None => return Err(ServerError::ConfigSSlKeyNone),
        };
        run_https(app, &listen_address, cert_path, key_path).await?;
    }
    log::info!("proxy server shutdown");
    Ok(())
}

fn build_app(config: Arc<ServerConfig>) -> Router {
    //静态文件夹
    let static_file_service =
        ServeDir::new("./web/public").fallback(ServeFile::new("./web/404.html"));
    //路由配置
    Router::new()
        .route(&config.path, routing::get(ws_handler_ns::ws_handler))
        .fallback(routing::get_service(static_file_service).handle_error(handle_error))
        .layer(Extension(config))
}

///等待停止信号
async fn wait_for_shutdown() {
    if let Err(e) = signal::ctrl_c().await {
        log::error!("wait stop signal failed: {e}");
    }
}

async fn run_http(app: Router, listen_address: &SocketAddr) -> Result<(), ServerError> {
    let builder = axum::Server::try_bind(listen_address)
        .map_err(|e| ServerError::Bind(listen_address.to_string(), e))?;
    log::info!("Server listen {listen_address} (ssl = false)");
    builder
        .serve(app.into_make_service())
        .with_graceful_shutdown(wait_for_shutdown())
        .await
        .map_err(ServerError::HttpService)
}

async fn run_https(
    app: Router,
    listen_address: &SocketAddr,
    cert_path: &str,
    key_path: &str,
) -> Result<(), ServerError> {
    let tls_config = RustlsConfig::from_pem_file(cert_path, key_path)
        .await
        .map_err(ServerError::Cert)?;
    let server = axum_server::bind_rustls(listen_address.to_owned(), tls_config);
    log::info!("Server listen {listen_address} (ssl = true)");
    tokio::select! {
        output1= server.serve(app.into_make_service()) =>output1.map_err(ServerError::HttpTlsService)?,
        _ = wait_for_shutdown() =>(),
    };
    Ok(())
}

async fn handle_error(_err: IoError) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
