extern crate dotenv;
use std::{sync::Arc, time::Duration};

use dotenv::dotenv;

use os_monitor::{detect_changes, initialize_monitor, Monitor};
use os_monitor_service::{db::db_manager, initialize_monitoring_service};
use tokio::{self, time::sleep};

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    println!("Starting os-monitor");
    log::info!("Starting os-monitor");
    dotenv().ok();

    let monitor = Arc::new(Monitor::new());
    let db_path = db_manager::get_default_db_path();
    initialize_monitoring_service(monitor.clone(), db_path).await;
    initialize_monitor(monitor.clone()).expect("Failed to initialize monitor");

    tokio::time::sleep(Duration::from_millis(350)).await;

    loop {
        sleep(Duration::from_secs(1)).await;
        detect_changes().expect("Failed to detect changes");
    }
}
