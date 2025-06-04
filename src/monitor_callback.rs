use os_monitor::{start_monitoring, Monitor};
use std::sync::Arc;
use std::time::Duration;

use crate::services;

pub struct MonitoringConfig {
    monitor: Arc<Monitor>,
    db_path: String,
    activity_state_interval: Duration,
}

impl MonitoringConfig {
    pub fn new(monitor: Arc<Monitor>, db_path: String) -> Self {
        Self {
            monitor,
            db_path,
            activity_state_interval: Duration::from_secs(60),
        }
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.activity_state_interval = interval;
        self
    }

    pub async fn initialize(self) {
        let activity_service =
            Arc::new(services::activities_service::start_activities_monitoring(self.db_path).await);
        activity_service
            .register_receiver(self.monitor.subscribe())
            .await;
        activity_service.start_activity_state_loop(self.activity_state_interval);

        std::thread::spawn(move || {
            start_monitoring(self.monitor);
        });
    }
}
