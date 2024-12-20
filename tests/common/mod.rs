use std::time::Duration;

use rafka_broker::Broker;
use rafka_storage::db::RetentionPolicy;
use tokio::task;

pub const DEFAULT_ADDRESS: &str = "127.0.0.1:50051";
pub const PORT: usize = 50051;
pub const PARTITION: usize = 0;
pub const TOTAL_PARTITIONS: usize = 1;

const ONE_GB: usize = 1024 * 1024 * 1024;

pub async fn setup_brokers(number_of_brokers: usize) {
    let retention_policy = RetentionPolicy {
        max_age: Duration::from_secs(1),
        max_bytes: ONE_GB,
    };

    for i in 0..number_of_brokers {
        let _ = task::spawn(async move {
            let address = &format!("127.0.0.1:{}", PORT + i);
            let broker = Broker::new(
                PARTITION as u32,
                TOTAL_PARTITIONS as u32,
                Some(retention_policy),
            );
            broker.serve(address).await.unwrap();
        });
    }
}
