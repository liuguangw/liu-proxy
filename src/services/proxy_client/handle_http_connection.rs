use super::check_server_conn::check_server_conn;
use super::run_proxy_tcp_loop::run_proxy_tcp_loop;
use super::server_conn_manger::ServerConnManger;
use super::stream::{StreamReader, StreamWriter};
use actix_web::web;
use bytes::Bytes;
use tokio::sync::mpsc::Sender;

///处理连接逻辑
pub async fn handle_connection(
    payload: web::Payload,
    tx: Sender<Result<Bytes, std::io::Error>>,
    target_addr: String,
    conn_manger: web::Data<ServerConnManger>,
) {
    /*
    let data = Bytes::from_static(b"666");
    tx.send(Ok(data.clone())).await.unwrap();
    time::sleep(Duration::from_secs(3)).await;
    tx.send(Ok(data)).await.unwrap();*/

    //向服务端发起websocket连接,并进行认证
    let mut ws_conn_pair = match conn_manger.get_conn_pair().await {
        Ok(s) => s,
        Err(e) => {
            log::error!("{e}");
            return;
        }
    };
    //把目标地址端口发给server,并检测server连接结果
    match check_server_conn(&mut ws_conn_pair, &target_addr).await {
        Ok(_) => {
            log::info!("server conn {target_addr} ok");
        }
        Err(e) => {
            log::error!("server conn {target_addr} failed: {e}");
            return;
        }
    };
    let mut stream_reader = StreamReader::Http(payload);
    let mut stream_writer = StreamWriter::Http(tx);
    if let Err(proxy_error) = run_proxy_tcp_loop(
        &conn_manger,
        ws_conn_pair,
        &mut stream_reader,
        &mut stream_writer,
    )
    .await
    {
        log::error!("{proxy_error}");
    }
}
