extern crate dotenv;
use std::sync::Arc;

use dotenv::dotenv;

use os_monitor::{
    detect_changes, get_application_icon_data, has_accessibility_permissions,
    request_accessibility_permissions, Monitor,
};
use os_monitor_service::{db::db_manager, initialize_monitoring_service};
use tokio::{self};

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    println!("Starting os-monitor");
    log::trace!("Starting os-monitor");
    dotenv().ok();

    let has_permissions = has_accessibility_permissions();
    println!("has_permissions: {}", has_permissions);
    if !has_permissions {
        let request_permissions = request_accessibility_permissions();
        println!("request_permissions: {}", request_permissions);
    }

    let monitor = Arc::new(Monitor::new());
    let db_path = db_manager::get_default_db_path();

    let icon_data = get_application_icon_data("com.apple.finder");
    println!("icon_data: {}", icon_data.unwrap());

    tokio::spawn(async move {
        initialize_monitoring_service(monitor.clone(), db_path).await;
    });

    std::thread::spawn(move || {
        // initialize_monitor(monitor_clone).expect("Failed to initialize monitor");
        loop {
            detect_changes().expect("Failed to detect changes");
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
