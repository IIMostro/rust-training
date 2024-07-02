use std::fmt::{Display, Formatter};
use log;
use tracing::{debug, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Default)]
struct User{
    name: String,
    age: u8,
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "name: {}, age: {}", self.name, self.age)
    }
}

async fn logging() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .init();
    let user = User::default();
    info!("Hello, world! {user}");
    debug!("This is a debug message");
    info!("Hello from tracing");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_logging() {
        logging().await;
    }
}