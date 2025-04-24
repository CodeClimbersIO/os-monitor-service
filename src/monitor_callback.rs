use os_monitor::{start_monitoring, Monitor};
use std::sync::Arc;
use std::time::Duration;

use crate::services;

static ACTIVITY_STATE_INTERVAL: Duration = Duration::from_secs(30); // every 30 seconds

pub async fn initialize_monitoring_service(monitor: Arc<Monitor>, db_path: String) {
    let activity_service =
        Arc::new(services::activities_service::start_activities_monitoring(db_path).await);
    activity_service
        .register_receiver(monitor.subscribe())
        .await;
    activity_service.start_activity_state_loop(ACTIVITY_STATE_INTERVAL);

    std::thread::spawn(move || {
        start_monitoring(monitor);
    });
}
