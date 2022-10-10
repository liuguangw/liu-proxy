use super::super::{proxy_error::ProxyError, send_message};
use super::build_request;
use super::parse_request::parse_request_body_size;
use crate::common::msg::{client::ProxyRequest, ClientMessage};
use crate::services::read_raw_data;
use bytes::{BufMut, Bytes, BytesMut};
use futures_util::Sink;
use httparse::Status;
use std::io::ErrorKind;
use tokio::io::AsyncRead;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};

///将客户端请求转发给server
pub async fn read_request_loop<T, U>(
    stream_reader: &mut T,
    ws_writer: &mut U,
    first_request_data: Bytes,
    remain_data_size: usize,
) -> Result<(), ProxyError>
where
    T: AsyncRead + Unpin,
    U: Sink<Message, Error = WsError> + Unpin,
{
    //发送第一个请求
    let is_disconn = send_request_data(
        stream_reader,
        ws_writer,
        first_request_data,
        remain_data_size,
    )
    .await?;
    if is_disconn {
        return Ok(());
    }
    //循环读取剩余的请求,并发送
    loop {
        let is_disconn = process_request(stream_reader, ws_writer).await?;
        if is_disconn {
            break;
        }
    }
    Ok(())
}

///从stream_reader读取新数据,如果连接被断开则会得到None,其他错误则是Err
async fn read_new_buf<T, U>(
    stream_reader: &mut T,
    ws_writer: &mut U,
) -> Result<Option<Bytes>, ProxyError>
where
    T: AsyncRead + Unpin,
    U: Sink<Message, Error = WsError> + Unpin,
{
    match read_raw_data::read_raw(stream_reader).await {
        Ok(data) => Ok(Some(data)),
        Err(e) => {
            //proxy被断开,通知服务端断开remote
            let disconn_msg = ClientMessage::DisConn;
            send_message::send_message(ws_writer, disconn_msg)
                .await
                .map_err(ProxyError::SendRequest)?;
            if e.kind() == ErrorKind::UnexpectedEof {
                //被主动断开
                Ok(None)
            } else {
                //因为读取错误而断开
                Err(ProxyError::ReadRequest(e))
            }
        }
    }
}

async fn send_request_data<T, U>(
    stream_reader: &mut T,
    ws_writer: &mut U,
    request_data: Bytes,
    mut remain_data_size: usize,
) -> Result<bool, ProxyError>
where
    T: AsyncRead + Unpin,
    U: Sink<Message, Error = WsError> + Unpin,
{
    //发送request的数据
    let request_msg = ProxyRequest(request_data);
    send_message::send_message(ws_writer, request_msg)
        .await
        .map_err(ProxyError::SendRequest)?;
    //发送剩余的数据
    while remain_data_size > 0 {
        let raw_data = match read_new_buf(stream_reader, ws_writer).await? {
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
        let request_msg = ProxyRequest(raw_data);
        send_message::send_message(ws_writer, request_msg)
            .await
            .map_err(ProxyError::SendRequest)?;
    }
    //正常,未被断开
    Ok(false)
}

async fn process_request<T, U>(stream_reader: &mut T, ws_writer: &mut U) -> Result<bool, ProxyError>
where
    T: AsyncRead + Unpin,
    U: Sink<Message, Error = WsError> + Unpin,
{
    let mut buf = BytesMut::new();
    loop {
        //读取数据
        let raw_data = match read_new_buf(stream_reader, ws_writer).await? {
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
            log::info!("proxy {} {}", req.method.unwrap(), req.path.unwrap());
            let request_data = build_request::build_request_data(&req, &buf, body_offset);
            let body_total_size = parse_request_body_size(req.headers)?;
            let remain_data_size = if body_total_size > 0 {
                body_total_size - (buf.len() - body_offset)
            } else {
                0
            };
            return send_request_data(stream_reader, ws_writer, request_data, remain_data_size)
                .await;
        }
    }
}
