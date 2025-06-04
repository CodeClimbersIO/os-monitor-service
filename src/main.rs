extern crate dotenv;
use std::{sync::Arc, time::Duration};

use dotenv::dotenv;

use os_monitor::{
    detect_changes, get_application_icon_data, has_accessibility_permissions,
    request_accessibility_permissions, Monitor,
};
use os_monitor_service::{db::db_manager, MonitoringConfig};
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

    let monitor = Monitor::new();

    let db_path = db_manager::get_default_db_path();

    let icon_data = get_application_icon_data("com.apple.finder");
    println!("icon_data: {}", icon_data.unwrap());

    tokio::spawn(async move {
        MonitoringConfig::new(Arc::new(monitor), db_path)
            .with_interval(Duration::from_secs(60))
            .initialize()
            .await;
    });

    std::thread::spawn(move || loop {
        detect_changes().expect("Failed to detect changes");
        std::thread::sleep(std::time::Duration::from_secs(1));
    });
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
