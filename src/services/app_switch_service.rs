use std::time::{Duration, SystemTime};

use crate::db::models::Activity;

pub struct AppSwitchState {
    current_activity: Option<Activity>,
    pub app_switches: i64,
    last_switch_time: SystemTime,
    min_time_between_switches: Duration,
}

impl AppSwitchState {
    pub fn new(min_time_between_switches: Duration) -> Self {
        AppSwitchState {
            current_activity: None,
            app_switches: 0,
            last_switch_time: SystemTime::now(),
            min_time_between_switches,
        }
    }

    pub fn new_window_activity(&mut self, activity: Activity) {
        if let Some(current_activity) = self.current_activity.clone() {
            if current_activity.app_id != activity.app_id {
                let now = SystemTime::now();
                if now
                    .duration_since(self.last_switch_time)
                    .unwrap_or(Duration::from_secs(0))
                    >= self.min_time_between_switches
                {
                    self.app_switches += 1;
                    self.last_switch_time = now;
                }
                self.current_activity = Some(activity);
            }
        } else {
            self.current_activity = Some(activity);
            self.last_switch_time = SystemTime::now();
        }
    }

    pub fn reset_app_switches(&mut self) {
        self.app_switches = 0;
    }
}

#[cfg(test)]
mod tests {
    use time::OffsetDateTime;
    use uuid::Uuid;

    use crate::db::{models::ActivityType, types::Platform};

    use super::*;

    #[test]
    fn test_app_switch() {
        let mut app_switch_state = AppSwitchState::new(Duration::from_millis(1));
        let app_1_id = Uuid::new_v4().to_string();
        let app_2_id = Uuid::new_v4().to_string();
        app_switch_state.new_window_activity(Activity::new(
            ActivityType::Window,
            Some("window1".to_string()),
            OffsetDateTime::now_utc(),
            Platform::Mac,
            Some(app_1_id.clone()),
        ));
        std::thread::sleep(Duration::from_millis(2));
        app_switch_state.new_window_activity(Activity::new(
            ActivityType::Window,
            Some("window2".to_string()),
            OffsetDateTime::now_utc(),
            Platform::Mac,
            Some(app_1_id.clone()),
        ));
        std::thread::sleep(Duration::from_millis(2));
        app_switch_state.new_window_activity(Activity::new(
            ActivityType::Window,
            Some("window3".to_string()),
            OffsetDateTime::now_utc(),
            Platform::Mac,
            Some(app_1_id.clone()),
        ));
        std::thread::sleep(Duration::from_millis(2));
        app_switch_state.new_window_activity(Activity::new(
            ActivityType::Window,
            Some("window4".to_string()),
            OffsetDateTime::now_utc(),
            Platform::Mac,
            Some(app_2_id.clone()),
        ));
        std::thread::sleep(Duration::from_millis(2));
        app_switch_state.new_window_activity(Activity::new(
            ActivityType::Window,
            Some("window5".to_string()),
            OffsetDateTime::now_utc(),
            Platform::Mac,
            Some(app_1_id.clone()),
        ));
        assert_eq!(app_switch_state.app_switches, 2);
    }

    #[test]
    fn test_spam_app_switch() {
        let mut app_switch_state = AppSwitchState::new(Duration::from_millis(1));
        for i in 0..5 {
            app_switch_state.new_window_activity(Activity::new(
                ActivityType::Window,
                Some(format!("window{}", i)),
                OffsetDateTime::now_utc(),
                Platform::Mac,
                Some(Uuid::new_v4().to_string()),
            ));
        }
        assert_eq!(app_switch_state.app_switches, 0);
    }

    #[test]
    fn test_app_switch_reset() {
        let mut app_switch_state = AppSwitchState::new(Duration::from_millis(1));
        app_switch_state.new_window_activity(Activity::new(
            ActivityType::Window,
            Some("window1".to_string()),
            OffsetDateTime::now_utc(),
            Platform::Mac,
            Some(Uuid::new_v4().to_string()),
        ));
        std::thread::sleep(Duration::from_millis(2));
        app_switch_state.new_window_activity(Activity::new(
            ActivityType::Window,
            Some("window2".to_string()),
            OffsetDateTime::now_utc(),
            Platform::Mac,
            Some(Uuid::new_v4().to_string()),
        ));
        app_switch_state.reset_app_switches();
        assert_eq!(app_switch_state.app_switches, 0);
    }
}
