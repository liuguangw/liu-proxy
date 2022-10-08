use bytes::{BufMut, Bytes, BytesMut};
use serde::Deserialize;
use sha1::{Digest, Sha1};

const TOKEN_SALT: &str = "9a340544-8f74-4d1e-b3b0-32a769615902";

#[derive(Deserialize, Debug, Clone)]
///授权用户
pub struct AuthUser {
    pub user: String,
    pub key: String,
}

impl AuthUser {
    ///根据时间戳计算token
    pub fn get_token(&self, ts: u64) -> Bytes {
        let mut buf = BytesMut::with_capacity(self.key.len() + 8 + TOKEN_SALT.len());
        buf.put_slice(self.key.as_bytes());
        buf.put_u64(ts);
        buf.put_slice(TOKEN_SALT.as_bytes());
        let mut hasher = Sha1::new();
        // process input message
        hasher.update(&buf);
        let result = hasher.finalize();
        let mut token_bytes = BytesMut::with_capacity(20);
        token_bytes.put_slice(&result);
        token_bytes.into()
    }
}
