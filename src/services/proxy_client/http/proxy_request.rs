use super::super::{poll_message, proxy_error::ProxyError, server_conn_manger::ConnPair};
use super::read_request_loop::read_request_loop;
use crate::common::msg::{server::ProxyResponseResult, ServerMessage};
use bytes::Bytes;
use futures_util::Stream;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};

pub async fn proxy_request(
    ws_conn_pair: &mut ConnPair,
    stream: &mut TcpStream,
    first_request_data: Bytes,
    remain_data_size: usize,
) -> Result<(), ProxyError> {
    let (mut stream_reader, mut stream_writer) = stream.split();
    tokio::select! {
        request_result = read_request_loop(&mut stream_reader, &mut ws_conn_pair.0,first_request_data,remain_data_size)=>request_result,
        response_result = read_response_loop(&mut ws_conn_pair.1,&mut  stream_writer)=>response_result,
    }
}

///将响应给客户端
async fn read_response_loop<T, U>(
    ws_reader: &mut T,
    stream_writer: &mut U,
) -> Result<(), ProxyError>
where
    T: Stream<Item = Result<Message, WsError>> + Unpin,
    U: AsyncWrite + Unpin,
{
    loop {
        let message = poll_message::poll_message(ws_reader)
            .await
            .map_err(ProxyError::PollMessage)?;
        let mut response_data = match message {
            //类型错误, 此时不应该收到这种消息
            ServerMessage::ConnResult(_) => return Err(ProxyError::InvalidServerMessage),
            ServerMessage::ResponseResult(response_result) => match response_result {
                //得到response
                ProxyResponseResult::Ok(s) => s,
                //server读取response失败
                ProxyResponseResult::Err(e) => return Err(ProxyError::ServerResponse(e)),
                //远端关闭了与server之间的连接
                ProxyResponseResult::Closed => break,
            },
            //server发送request到远端失败
            ServerMessage::RequestFail(e) => return Err(ProxyError::ServerRequest(e.0)),
        };
        //dbg!(&response_data);
        if let Err(e) = stream_writer.write_all_buf(&mut response_data).await {
            return Err(ProxyError::WriteResponse(e));
        }
    }
    Ok(())
}
