extern crate dotenv;
use std::{sync::Arc, time::Duration};

use dotenv::dotenv;

use os_monitor::{detect_changes, initialize_monitor, Monitor};
use os_monitor_service::{enable_log, initialize_monitoring_service, log};
use tokio::{self, time::sleep};

#[tokio::main]
async fn main() {
    enable_log();
    log("starting activity service");
    dotenv().ok();

    let monitor = Arc::new(Monitor::new());
    initialize_monitoring_service(monitor.clone()).await;
    initialize_monitor(monitor.clone()).expect("Failed to initialize monitor");

    tokio::time::sleep(Duration::from_millis(350)).await;

    loop {
        sleep(Duration::from_secs(1)).await;
        detect_changes().expect("Failed to detect changes");
    }
}
