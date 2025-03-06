#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn future_select_should_work() {
        let mut i = 0;
        while i < 10 {
            let future1 = async {
                tokio::time::sleep(Duration::from_secs(1)).await;
                println!("Task {i} done");
            };
        }
    }

    #[tokio::test]
    async fn test_get_current_timestamp() {
        // 每日结束后压缩日志
        let now = SystemTime::now();
        let midnight = now.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() / 86400 * 86400;
        let current_day_start = SystemTime::UNIX_EPOCH + Duration::new(midnight, 0);
        let path = Path::new("/home/neptune/logs");
        for entry in fs::read_dir(path).unwrap() {
            let file_created_time = entry.unwrap().path().metadata().unwrap().created().unwrap();
            println!("{}", file_created_time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs())
        }
        println!("Current Micro Seconds: {}", now.duration_since(UNIX_EPOCH).unwrap().as_secs());
        println!("Current Nano Seconds:  {}", midnight);
        println!("Current Seconds: {}", current_day_start.duration_since(UNIX_EPOCH).unwrap().as_secs());
        // println!("Current Seconds in f64: {}", time.as_secs_f64());
    }
}