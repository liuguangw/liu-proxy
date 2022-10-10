use bytes::{BufMut, Bytes, BytesMut};
use http::Uri;
use httparse::Request;

///从req中构造request bytes
pub fn build_request_data(req: &Request<'_, '_>, buf: &[u8], body_offset: usize) -> Bytes {
    let req_method = req.method.unwrap();
    let request_uri: Uri = req.path.unwrap().parse().unwrap();
    let http_version = if req.version.unwrap() == 1 {
        "1.1"
    } else {
        "1.0"
    };
    //只需要path_and_query
    let request_line = format!(
        "{req_method} {} HTTP/{http_version}\r\n",
        request_uri.path_and_query().unwrap()
    );
    let mut data_bytes = BytesMut::from(request_line.as_bytes());
    //写入http header
    for header_info in req.headers.iter() {
        data_bytes.put_slice(header_info.name.as_bytes());
        data_bytes.put_slice(b": ");
        data_bytes.put_slice(header_info.value);
        data_bytes.put_slice(b"\r\n");
    }
    data_bytes.put_slice(b"\r\n");
    //写入body部分
    if body_offset < buf.len() {
        data_bytes.put_slice(&buf[body_offset..]);
    }
    data_bytes.into()
}
