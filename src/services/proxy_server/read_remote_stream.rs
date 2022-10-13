use super::proxy_error::ProxyError;
use crate::common::msg::{server::ProxyResponseResult, ServerMessage};
use crate::services::read_raw_data;
use std::io::ErrorKind;
use tokio::{net::tcp::ReadHalf, sync::mpsc::Sender};

pub async fn read_remote_stream(
    mut remote_reader: ReadHalf<'_>,
    tx: Sender<ServerMessage>,
) -> Result<(), ProxyError> {
    loop {
        let mut read_response_ok = true;
        let response_result_msg = match read_raw_data::read_raw(&mut remote_reader).await {
            Ok(data) => ProxyResponseResult::Ok(data),
            Err(e) => {
                read_response_ok = false;
                if e.kind() == ErrorKind::UnexpectedEof {
                    ProxyResponseResult::Closed
                } else {
                    ProxyResponseResult::Err(e.to_string())
                }
            }
        };
        //把read远端的结果发给客户端
        tx.send(response_result_msg.into())
            .await
            .map_err(|_| ProxyError::WriteChannel)?;
        //response失败跳出循环
        if !read_response_ok {
            break;
        }
    }
    //log::info!("read_remote_stream end");
    Ok(())
}
