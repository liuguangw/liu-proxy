use super::super::proxy_error::ProxyError;
use super::build_request;
use super::parse_request::parse_request_body_size;
use crate::services::proxy_client::connection::ConnWriter;
use crate::services::read_raw_data;
use bytes::{BufMut, Bytes, BytesMut};
use httparse::Status;
use std::io::ErrorKind;
use tokio::io::AsyncRead;

///将客户端请求转发给server
pub async fn read_request_loop<T>(
    stream_reader: &mut T,
    remote_conn_writer: &mut ConnWriter<'_>,
    first_request_data: Bytes,
    remain_data_size: usize,
) -> Result<(), ProxyError>
where
    T: AsyncRead + Unpin,
{
    //发送第一个请求
    let is_disconn = send_request_data(
        stream_reader,
        remote_conn_writer,
        first_request_data,
        remain_data_size,
    )
    .await?;
    if is_disconn {
        return Ok(());
    }
    //循环读取剩余的请求,并发送
    loop {
        let is_disconn = process_request(stream_reader, remote_conn_writer).await?;
        if is_disconn {
            break;
        }
    }
    Ok(())
}

///从stream_reader读取新数据,如果连接被断开则会得到None,其他错误则是Err
async fn read_new_buf<T>(
    stream_reader: &mut T,
    remote_conn_writer: &mut ConnWriter<'_>,
) -> Result<Option<Bytes>, ProxyError>
where
    T: AsyncRead + Unpin,
{
    match read_raw_data::read_raw(stream_reader).await {
        Ok(data) => Ok(Some(data)),
        Err(e) => {
            //proxy被断开,通知服务端断开remote
            remote_conn_writer.process_client_close().await?;
            let err_kind = e.kind();
            if err_kind == ErrorKind::UnexpectedEof || err_kind == ErrorKind::ConnectionAborted {
                //被主动断开
                Ok(None)
            } else {
                //因为读取错误而断开
                //dbg!(&e);
                Err(ProxyError::ReadRequest(e))
            }
        }
    }
}

async fn send_request_data<T>(
    stream_reader: &mut T,
    remote_conn_writer: &mut ConnWriter<'_>,
    request_data: Bytes,
    mut remain_data_size: usize,
) -> Result<bool, ProxyError>
where
    T: AsyncRead + Unpin,
{
    //发送request的数据
    remote_conn_writer.write_data(request_data).await?;
    //发送剩余的数据
    while remain_data_size > 0 {
        let raw_data = match read_new_buf(stream_reader, remote_conn_writer).await? {
            Some(s) => s,
            //被主动断开
            None => return Ok(true),
        };
        if raw_data.len() <= remain_data_size {
            remain_data_size -= raw_data.len();
        } else {
            //在前一个http请求没有完全发送出去的情况下,不可能接着发送第二个
            log::error!("bad request overflow");
            remain_data_size = 0;
        }
        remote_conn_writer.write_data(raw_data).await?;
    }
    //正常,未被断开
    Ok(false)
}

async fn process_request<T>(
    stream_reader: &mut T,
    remote_conn_writer: &mut ConnWriter<'_>,
) -> Result<bool, ProxyError>
where
    T: AsyncRead + Unpin,
{
    let mut buf = BytesMut::new();
    loop {
        //读取数据
        let raw_data = match read_new_buf(stream_reader, remote_conn_writer).await? {
            Some(s) => s,
            //被主动断开
            None => return Ok(true),
        };
        buf.put_slice(&raw_data);
        //解析
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);
        let offset_status = req.parse(&buf)?;
        //读取到了完整的http头信息
        if let Status::Complete(body_offset) = offset_status {
            //log::info!("proxy {} {}", req.method.unwrap(), req.path.unwrap());
            let request_data = build_request::build_request_data(&req, &buf, body_offset);
            let body_total_size = parse_request_body_size(req.headers)?;
            let remain_data_size = if body_total_size > 0 {
                body_total_size - (buf.len() - body_offset)
            } else {
                0
            };
            return send_request_data(
                stream_reader,
                remote_conn_writer,
                request_data,
                remain_data_size,
            )
            .await;
        }
    }
}
