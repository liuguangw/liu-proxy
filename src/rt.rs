use std::future::Future;
use tokio::runtime::Runtime;

/// 构造异步运行时
pub fn runtime() -> Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .thread_name("tokio_main_rt")
        .enable_all()
        .build()
        .expect("create tokio runtime")
}

/// 执行异步代码
pub fn block_on<F: Future>(future: F) -> F::Output {
    runtime().block_on(future)
}
