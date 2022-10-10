use std::future::Future;
use tokio::runtime::Runtime;

/// 构造异步运行时
pub fn runtime(worker_count_opt: Option<usize>) -> Runtime {
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    if let Some(worker_count) = worker_count_opt {
        builder.worker_threads(worker_count);
    };
    builder
        .thread_name("tokio_main_rt")
        .enable_all()
        .build()
        .expect("create tokio runtime")
}

/// 执行异步代码
pub fn block_on<F: Future>(future: F) -> F::Output {
    runtime(None).block_on(future)
}
