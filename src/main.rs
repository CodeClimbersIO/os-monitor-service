extern crate dotenv;
use std::{sync::Arc, time::Duration};

use dotenv::dotenv;

use os_monitor::{
    detect_changes, has_accessibility_permissions, initialize_monitor,
    request_accessibility_permissions, Monitor,
};
use os_monitor_service::{db::db_manager, initialize_monitoring_service};
use tokio::{self, time::sleep};

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv().ok();

    let has_permissions = has_accessibility_permissions();
    println!("has_permissions: {}", has_permissions);
    if !has_permissions {
        let request_permissions = request_accessibility_permissions();
        println!("request_permissions: {}", request_permissions);
    }

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
