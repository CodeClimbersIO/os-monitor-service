use os_monitor::{KeyboardEvent, MouseEvent, WindowEvent};
use sqlx::Row;
use time::OffsetDateTime;

use crate::db::types::Platform;

#[derive(Debug, sqlx::Type, PartialEq, Clone)]
#[sqlx(type_name = "TEXT", rename_all = "UPPERCASE")]
pub enum ActivityType {
    Keyboard,
    Mouse,
    Window,
}

impl From<String> for ActivityType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "MOUSE" => ActivityType::Mouse,
            "KEYBOARD" => ActivityType::Keyboard,
            "WINDOW" => ActivityType::Window,
            _ => panic!("Unknown activity type: {}", s), // Or handle invalid types differently
        }
    }
}

#[derive(Clone, Debug)]
pub struct Activity {
    pub id: Option<i64>,
    pub created_at: Option<OffsetDateTime>,
    pub timestamp: Option<OffsetDateTime>,
    pub activity_type: ActivityType,
    pub app_window_title: Option<String>,
    pub platform: Platform,
    pub app_id: Option<String>,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for Activity {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Activity {
            id: row.try_get("id")?,
            created_at: row.try_get("created_at")?,
            timestamp: row.try_get("timestamp")?,
            activity_type: row.try_get("activity_type")?,
            app_window_title: row.try_get("app_window_title")?,
            platform: row.try_get("platform")?,
            app_id: row.try_get("app_id")?,
        })
    }
}

impl Activity {
    pub fn new(
        activity_type: ActivityType,
        app_window_title: Option<String>,
        timestamp: OffsetDateTime,
        platform: Platform,
        app_id: Option<String>,
    ) -> Self {
        Activity {
            id: None,
            created_at: Some(OffsetDateTime::now_utc()),
            timestamp: Some(timestamp),
            activity_type,
            app_window_title,
            platform,
            app_id,
        }
    }

    pub fn create_window_activity(event: &WindowEvent, app_id: Option<String>) -> Self {
        log::trace!("create_window_activity: {:?}", event);
        Self::new(
            ActivityType::Window,
            Some(event.window_title.clone()),
            OffsetDateTime::now_utc(),
            event.platform.into(),
            app_id,
        )
    }

    pub fn create_mouse_activity(_: &MouseEvent) -> Self {
        Self::new(
            ActivityType::Mouse,
            None,
            OffsetDateTime::now_utc(),
            Platform::Mac,
            None,
        )
    }

    pub fn create_keyboard_activity(_: &KeyboardEvent) -> Self {
        Self::new(
            ActivityType::Keyboard,
            None,
            OffsetDateTime::now_utc(),
            Platform::Mac,
            None,
        )
    }

    #[cfg(test)]
    pub fn __create_test_window(app_name: Option<String>, app_id: Option<String>) -> Self {
        use os_monitor::Platform;

        Self::create_window_activity(
            &WindowEvent {
                window_title: "main.rs - app-codeclimbers".to_string(),
                platform: Platform::Mac,
                app_name: app_name.unwrap_or("Cursor".to_string()),
                bundle_id: Some("com.ebb.app".to_string()),
                url: Some("https://cursor.com".to_string()),
            },
            app_id,
        )
    }
}
