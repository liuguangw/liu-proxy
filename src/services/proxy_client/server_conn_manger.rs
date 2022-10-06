use crate::common::{ClientConfig, NoServerCertVerifier, WebsocketRequest};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use rustls::ClientConfig as TlsClientConfig;
use std::{collections::VecDeque, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tokio::{net::TcpStream, time::timeout};
use tokio_tungstenite::{
    tungstenite::{handshake::server::Response, Error as WsError, Message},
    Connector, MaybeTlsStream, WebSocketStream,
};
///一对连接
pub type ConnPair = (
    SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
);

///服务端连接管理器
#[derive(Clone)]
pub struct ServerConnManger {
    ws_request: WebsocketRequest,
    max_idle_conns: u32,
    conn_pool: Arc<Mutex<VecDeque<ConnPair>>>,
}

impl ServerConnManger {
    pub fn try_init(
        config: &ClientConfig,
    ) -> Result<Self, <WebsocketRequest as TryFrom<&ClientConfig>>::Error> {
        let ws_request = WebsocketRequest::try_from(config)?;
        let conn_list = if config.max_idle_conns > 0 {
            VecDeque::with_capacity(config.max_idle_conns as usize)
        } else {
            VecDeque::new()
        };
        let conn_pool = Arc::new(Mutex::new(conn_list));
        Ok(Self {
            ws_request,
            max_idle_conns: config.max_idle_conns,
            conn_pool,
        })
    }

    ///处理客户端与服务端之间的握手操作
    async fn auth_handshake(
        &self,
    ) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), WsError> {
        //tls connector
        let connector = if self.ws_request.insecure {
            //跳过ssl证书验证
            let client_config = TlsClientConfig::builder()
                .with_safe_defaults()
                .with_custom_certificate_verifier(Arc::new(NoServerCertVerifier {}))
                .with_no_client_auth();
            let client_config = Arc::new(client_config);
            Some(Connector::Rustls(client_config))
        } else {
            //默认配置
            None
        };
        //建立tcp连接
        //log::info!("tcp conn: {}", ws_request.server_addr);
        let stream = TcpStream::connect(self.ws_request.server_addr)
            .await
            .map_err(WsError::Io)?;
        //websocket握手
        tokio_tungstenite::client_async_tls_with_config(&self.ws_request, stream, None, connector)
            .await
    }

    ///建立一个新的连接
    async fn create_new_conn(&self) -> Result<ConnPair, WsError> {
        let (stream, _) = self.auth_handshake().await?;
        let conn_pair = stream.split();
        Ok(conn_pair)
    }

    ///取出一个存在的连接
    async fn fetch_exist_conn(&self) -> Option<ConnPair> {
        let mut lock = self.conn_pool.lock().await;
        log::info!(
            "get_conn_pair(in lock): idle_conns={}, max_idle_conns={}",
            lock.len(),
            self.max_idle_conns
        );
        lock.pop_front()
    }

    ///检测取出的连接,判断是否有效
    async fn check_conn_status(&self, conn_pair: &mut ConnPair) -> Result<(), WsError> {
        let ws_writer = &mut conn_pair.0;
        //发送ping
        let data = vec![6, 6, 6];
        ws_writer.send(Message::Ping(data)).await?;
        //等待pong
        let ws_reader = &mut conn_pair.1;
        if let Some(message_result) = ws_reader.next().await {
            let message = message_result?;
            if let Message::Pong(_) = message {
                return Ok(());
            }
        }
        Err(WsError::AlreadyClosed)
    }

    ///取出连接
    pub async fn get_conn_pair(&self) -> Result<ConnPair, WsError> {
        if self.max_idle_conns == 0 {
            //不使用连接池
            return self.create_new_conn().await;
        }
        //指定每次ping测试时间限制
        let timeout_duration = Duration::from_millis(1500);
        while let Some(mut conn_pair) = self.fetch_exist_conn().await {
            //判断取出的连接是否有效
            let tm_check_result =
                timeout(timeout_duration, self.check_conn_status(&mut conn_pair)).await;
            if let Ok(Ok(_)) = tm_check_result {
                log::info!("get_conn_pair(from pool)");
                return Ok(conn_pair);
            }
        }
        log::info!("get_conn_pair(create new)");
        self.create_new_conn().await
    }

    async fn close_conn_pair(&self, conn: ConnPair) {
        let mut ws_writer = conn.0;
        if let Err(e) = ws_writer.close().await {
            log::error!("close ws conn failed: {e}");
        }
    }

    ///将连接放回
    pub async fn push_back_conn(&self, conn: ConnPair) {
        //不使用连接池
        if self.max_idle_conns == 0 {
            self.close_conn_pair(conn).await;
            return;
        }
        let mut lock = self.conn_pool.lock().await;
        let idle_conns = lock.len();
        //空闲连接已满
        if idle_conns >= (self.max_idle_conns as usize) {
            log::info!(
                "push_back_conn(full): idle_conns={}, max_idle_conns={}",
                idle_conns,
                self.max_idle_conns
            );
            self.close_conn_pair(conn).await;
            return;
        }
        log::info!(
            "push_back_conn(success): idle_conns={}, max_idle_conns={}",
            idle_conns + 1,
            self.max_idle_conns
        );
        lock.push_back(conn);
    }
}
