use super::super::proxy_error::ProxyError;
use super::read_request_loop::read_request_loop;
use crate::services::proxy_client::connection::{ConnReader, ConnWriter, ConnectionError};
use bytes::Bytes;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn proxy_request(
    remote_conn_writer: &mut ConnWriter<'_>,
    remote_conn_reader: &mut ConnReader<'_>,
    stream: &mut TcpStream,
    first_request_data: Bytes,
    remain_data_size: usize,
) -> Result<(), ProxyError> {
    let (mut stream_reader, mut stream_writer) = stream.split();
    tokio::select! {
        request_result = read_request_loop(&mut stream_reader, remote_conn_writer,first_request_data,remain_data_size)=>request_result,
        response_result = read_response_loop(remote_conn_reader,&mut  stream_writer)=>response_result,
    }
}

///将响应给客户端
async fn read_response_loop<T>(
    remote_conn_reader: &mut ConnReader<'_>,
    stream_writer: &mut T,
) -> Result<(), ProxyError>
where
    T: AsyncWrite + Unpin,
{
    loop {
        let mut response_data = match remote_conn_reader.read_data().await {
            Ok(s) => s,
            Err(e) => match e {
                ConnectionError::WsRemoteClosed | ConnectionError::ConnClosed => break,
                _ => return Err(e.into()),
            },
        };
        //dbg!(&response_data);
        if let Err(e) = stream_writer.write_all_buf(&mut response_data).await {
            return Err(ProxyError::WriteResponse(e));
        }
    }
    Ok(())
}
