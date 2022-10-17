use crate::common::{ClientConfig, ParseWebsocketRequestError, WebsocketRequest};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::{collections::VecDeque, sync::Arc, time::Duration};
use tokio::{net::TcpStream, time::timeout};
use tokio::{sync::Mutex, time};
use tokio_tungstenite::{
    tungstenite::{handshake::server::Response, Error as WsError, Message},
    MaybeTlsStream, WebSocketStream,
};
pub type ConnPairReader = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
pub type ConnPairWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
///一对连接
pub type ConnPair = (ConnPairWriter, ConnPairReader);

///服务端连接管理器
#[derive(Clone)]
pub struct ServerConnManger {
    ws_request: Arc<WebsocketRequest>,
    max_idle_conns: u32,
    conn_pool: Arc<Mutex<VecDeque<ConnPair>>>,
}

impl ServerConnManger {
    pub fn try_init(config: &ClientConfig) -> Result<Self, ParseWebsocketRequestError> {
        let ws_request = WebsocketRequest::try_from(config)?;
        let ws_request = Arc::new(ws_request);
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
        let connector = self.ws_request.ssl_connector.to_owned();
        //建立tcp连接
        //log::info!("tcp conn: {}", ws_request.server_addr);
        let stream = TcpStream::connect(self.ws_request.server_addr)
            .await
            .map_err(WsError::Io)?;
        //websocket握手
        tokio_tungstenite::client_async_tls_with_config(
            self.ws_request.as_ref(),
            stream,
            None,
            connector,
        )
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
        /*log::info!(
            "get_conn_pair(in lock): idle_conns={}, max_idle_conns={}",
            lock.len(),
            self.max_idle_conns
        );*/
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
        while let Some(message_result) = ws_reader.next().await {
            //log::info!("check1");
            //可能收到上一轮的二进制消息(远端关闭比用户关闭快),或者pong
            //dbg!(&message_result);
            let message = message_result?;
            if let Message::Pong(_) = message {
                return Ok(());
            }
        }
        //log::info!("check3");
        Err(WsError::AlreadyClosed)
    }

    ///取出连接
    pub async fn get_conn_pair(&self) -> Result<ConnPair, WsError> {
        if self.max_idle_conns == 0 {
            //不使用连接池
            return self.create_new_conn().await;
        }
        //指定每次ping测试时间限制
        let timeout_duration = Duration::from_millis(1800);
        //取一个连接,如果连接不可用就新建连接(不尝试连接池剩余的)
        if let Some(mut conn_pair) = self.fetch_exist_conn().await {
            //判断取出的连接是否有效
            let tm_check_result =
                timeout(timeout_duration, self.check_conn_status(&mut conn_pair)).await;
            //dbg!(&tm_check_result);
            if let Ok(Ok(_)) = tm_check_result {
                //log::info!("get_conn_pair(from pool)");
                return Ok(conn_pair);
            }
        }
        //log::info!("get_conn_pair(create new)");
        self.create_new_conn().await
    }

    async fn close_conn_pair(&self, conn: ConnPair) {
        let mut ws_writer = conn.0;
        if let Err(e) = ws_writer.close().await {
            log::error!("close ws conn failed: {e}");
        }
    }

    ///将空闲连接放回
    pub async fn push_back_conn(&self, conn: ConnPair) {
        //不使用连接池
        if self.max_idle_conns == 0 {
            self.close_conn_pair(conn).await;
            return;
        }
        let mut old_conn_opt = None;
        {
            let mut lock = self.conn_pool.lock().await;
            let idle_conns = lock.len();
            //dbg!(idle_conns,self.max_idle_conns);
            //空闲连接已满
            if idle_conns >= (self.max_idle_conns as usize) {
                //弹出前面最旧的连接
                old_conn_opt = lock.pop_front();
            }
            //把新连接放到后面
            lock.push_back(conn);
        }
        //关闭旧连接
        if let Some(old_conn) = old_conn_opt {
            self.close_conn_pair(old_conn).await;
        }
    }
    ///每隔一断时间检测一次空闲连接状态
    pub async fn scan_conn_pool(&self) {
        let mut interval = time::interval(Duration::from_secs(8));
        loop {
            interval.tick().await;
            //log::info!("scan start");
            let conn_count = {
                let lock = self.conn_pool.lock().await;
                lock.len()
            };
            //log::info!("conn total count={conn_count}");
            for _i in 0..conn_count {
                //log::info!("scan {}/{conn_count}", i + 1);
                if let Ok(conn) = self.get_conn_pair().await {
                    self.push_back_conn(conn).await;
                }
            }
            //log::info!("scan end");
        }
    }

    ///释放所有连接
    pub async fn clear_conns(&self) {
        while let Some(conn) = self.fetch_exist_conn().await {
            self.close_conn_pair(conn).await;
        }
    }
}
