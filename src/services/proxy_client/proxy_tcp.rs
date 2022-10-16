use super::connection::{ConnReader, ConnWriter, ConnectionError};
use super::proxy_error::ProxyError;
use crate::services::read_raw_data;
use std::io::ErrorKind;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn proxy_tcp(
    remote_conn_writer: &mut ConnWriter<'_>,
    remote_conn_reader: &mut ConnReader<'_>,
    stream: &mut TcpStream,
) -> Result<(), ProxyError> {
    let (mut stream_reader, mut stream_writer) = stream.split();
    tokio::select! {
        request_result = read_request_loop(&mut stream_reader, remote_conn_writer)=>request_result,
        response_result = read_response_loop(remote_conn_reader,&mut  stream_writer)=>response_result,
    }
}

///将客户端请求转发给server
async fn read_request_loop<T>(
    stream_reader: &mut T,
    remote_conn_writer: &mut ConnWriter<'_>,
) -> Result<(), ProxyError>
where
    T: AsyncRead + Unpin,
{
    loop {
        //读取客户端请求
        let raw_data = match read_raw_data::read_raw(stream_reader).await {
            Ok(data) => data,
            Err(e) => {
                //proxy被断开,通知服务端断开remote
                remote_conn_writer.process_client_close().await?;
                let err_kind = e.kind();
                if err_kind == ErrorKind::UnexpectedEof || err_kind == ErrorKind::ConnectionAborted
                {
                    //被主动断开
                    break;
                } else {
                    //因为读取错误而断开
                    //dbg!(&e);
                    return Err(ProxyError::ReadRequest(e));
                }
            }
        };
        //发送请求
        remote_conn_writer.write_data(raw_data).await?;
    }
    Ok(())
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
