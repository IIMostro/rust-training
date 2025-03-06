#[cfg(test)]
mod tests {
    use anyhow::Result;
    use tracing_subscriber::{EnvFilter, fmt};
    use tracing_subscriber::fmt::time::ChronoLocal;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    #[tokio::test]
    async fn test_console_should_work() -> Result<()> {
        let console_layer = console_subscriber::ConsoleLayer::builder().server_addr(([0,0,0,0], 6669)).spawn();
        let timer = ChronoLocal::new("%Y-%m-%d %H:%M:%S".to_string());
        let filter_layer = EnvFilter::new("TRACE");
        tracing_subscriber::registry()
            .with(console_layer)
            .with(fmt::layer() // 配置 span 事件
                .with_timer(timer)
                .with_target(false) // 关闭目标日志源显示
                .with_thread_ids(true) // 显示线程 ID
                .with_thread_names(false))
            .with(filter_layer)
            .init();
        // 示例任务
        tokio::spawn(async {
            for i in 0..10 {
                println!("Task 1 iteration: {}", i);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });
        tokio::spawn(async {
            for i in 0..10 {
                println!("Task 2 iteration: {}", i);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });
        // 主任务等待一段时间
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        Ok(())
    }
}