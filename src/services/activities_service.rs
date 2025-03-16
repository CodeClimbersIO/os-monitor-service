use std::{sync::Arc, time::Duration};

use once_cell::sync::Lazy;
use os_monitor::{BlockedAppEvent, KeyboardEvent, Monitor, MouseEvent, WindowEvent};
use parking_lot::Mutex;
use tokio::sync::mpsc;

use crate::db::{
    activity_repo::ActivityRepo,
    activity_state_repo::ActivityStateRepo,
    blocked_activity_repo::BlockedActivityRepo,
    db_manager,
    models::{Activity, BlockedActivity},
};

use self::activity_state_service::ActivityPeriod;

use super::{
    activity_state_service::{self, ActivityStateService},
    app_service::AppService,
    app_switch_service::AppSwitchState,
};

#[cfg(test)]
use crate::db::models::ActivityState;
#[cfg(test)]
use time::OffsetDateTime;

static APP_SWITCH_STATE: Lazy<Mutex<AppSwitchState>> =
    Lazy::new(|| Mutex::new(AppSwitchState::new(Duration::from_secs(2))));

#[derive(Clone)]
pub struct ActivityService {
    activities_repo: ActivityRepo,
    activity_state_repo: ActivityStateRepo,
    app_service: AppService,
    event_sender: mpsc::UnboundedSender<ActivityEvent>,
    activity_state_service: ActivityStateService,
    blocked_activity_repo: BlockedActivityRepo,
}

#[derive(Debug)]
enum ActivityEvent {
    Keyboard(KeyboardEvent),
    Mouse(MouseEvent),
    Window(WindowEvent),
    AppBlocked(BlockedAppEvent),
}

impl ActivityService {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let activities_repo = ActivityRepo::new(pool.clone());
        let activity_state_repo = ActivityStateRepo::new(pool.clone());
        let activity_state_service = ActivityStateService::new(pool.clone());
        let app_service = AppService::new(pool.clone());
        let blocked_activity_repo = BlockedActivityRepo::new(pool.clone());
        let service = ActivityService {
            activities_repo,
            activity_state_repo,
            app_service,
            event_sender: sender,
            activity_state_service,
            blocked_activity_repo,
        };
        let callback_service_clone = service.clone();
        // let activity_state_clone = service.clone();

        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                match event {
                    ActivityEvent::Keyboard(event) => {
                        callback_service_clone.handle_keyboard_activity(event).await
                    }
                    ActivityEvent::Mouse(event) => {
                        callback_service_clone.handle_mouse_activity(event).await
                    }
                    ActivityEvent::Window(event) => {
                        callback_service_clone.handle_window_activity(event).await
                    }
                    ActivityEvent::AppBlocked(event) => {
                        callback_service_clone
                            .handle_app_blocked_activity(event)
                            .await
                    }
                }
            }
        });

        service
    }

    async fn handle_keyboard_activity(&self, event: KeyboardEvent) {
        log::trace!("{}: {:?}", "handle_keyboard_activity", event);
        let activity = Activity::create_keyboard_activity(&event);
        if let Err(err) = self.save_activity(&activity).await {
            log::error!("Failed to save keyboard activity: {}", err);
        }
    }

    async fn handle_mouse_activity(&self, event: MouseEvent) {
        log::trace!("{}: {:?}", "handle_mouse_activity", event);
        let activity = Activity::create_mouse_activity(&event);
        if let Err(err) = self.save_activity(&activity).await {
            log::error!("Failed to save mouse activity: {}", err);
        }
    }

    pub async fn handle_window_activity(&self, event: WindowEvent) {
        log::trace!("{}: {:?}", "handle_window_activity", event);
        let app_id = self.app_service.handle_window_event(&event).await;
        if let Ok(app_id) = app_id {
            let activity = Activity::create_window_activity(&event, Some(app_id));
            if let Err(err) = self.save_activity(&activity).await {
                log::error!("Failed to save window activity: {}", err);
            }
            let mut app_switch_state = APP_SWITCH_STATE.lock();
            app_switch_state.new_window_activity(activity);
        } else {
            log::error!("Failed to get or create app id");
        }
    }

    async fn handle_app_blocked_activity(&self, event: BlockedAppEvent) {
        log::trace!("{}: {:?}", "handle_app_blocked_activity", event);

        for blocked_app in event.blocked_apps {
            let blocked_activity = BlockedActivity {
                id: uuid::Uuid::new_v4().to_string(),
                external_app_id: blocked_app.app_external_id,
                created_at: Some(time::OffsetDateTime::now_utc()),
                updated_at: Some(time::OffsetDateTime::now_utc()),
            };

            if let Err(err) = self
                .blocked_activity_repo
                .save_blocked_activity(&blocked_activity)
                .await
            {
                log::error!("Failed to save blocked activity: {}", err);
            }
        }
    }

    pub fn register_callbacks(&self, event_callback_service: &Arc<Monitor>) {
        let sender = self.event_sender.clone();
        event_callback_service.register_keyboard_callback(Box::new(move |event| {
            if event {
                let _ = sender.send(ActivityEvent::Keyboard(KeyboardEvent {}));
            }
        }));

        let sender = self.event_sender.clone();
        event_callback_service.register_mouse_callback(Box::new(move |event| {
            if event {
                let _ = sender.send(ActivityEvent::Mouse(MouseEvent {}));
            }
        }));

        let sender = self.event_sender.clone();
        event_callback_service.register_window_callback(Box::new(move |event| {
            log::trace!("register_window_callback");
            let _ = sender.send(ActivityEvent::Window(event));
        }));
        let sender = self.event_sender.clone();
        event_callback_service.register_app_blocked_callback(Box::new(move |event| {
            log::trace!("register_app_blocked_callback");
            let _ = sender.send(ActivityEvent::AppBlocked(event));
        }));
    }

    pub async fn save_activity(
        &self,
        activity: &Activity,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        self.activities_repo.save_activity(activity).await
    }

    #[cfg(test)]
    pub async fn get_activity(&self, id: i32) -> Result<Activity, sqlx::Error> {
        self.activities_repo.get_activity(id).await
    }

    #[cfg(test)]
    async fn save_activity_state(
        &self,
        activity_state: &ActivityState,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        self.activity_state_repo
            .save_activity_state(activity_state)
            .await
    }

    pub async fn get_activities_since_last_activity_state(
        &self,
    ) -> Result<Vec<Activity>, sqlx::Error> {
        self.activities_repo
            .get_activities_since_last_activity_state()
            .await
    }

    async fn create_idle_activity_state(
        &self,
        activity_period: ActivityPeriod,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        self.activity_state_repo
            .create_idle_activity_state(&activity_period)
            .await
            .expect("Failed to create idle activity state");

        let activity_state = self
            .activity_state_service
            .get_last_activity_state()
            .await
            .expect("Failed to get activity state");

        if let Some(activity_state_id) = activity_state.id {
            self.app_service.create_idle_tag(activity_state_id).await
        } else {
            log::error!("Cannot create tags: activity state has no ID");
            Err(sqlx::Error::RowNotFound)
        }
    }

    /**
     * Creates an activity state from a list of activities.
     * If the activities are empty, it creates an idle activity state.
     * It also creates an idle tag for the idle activity state.
     * If the activities are not empty, it creates an active activity state.
     * For tags, we get all matching tags for the activites and create those tags for the activity state.
     * If there were no window activities, we use the last window activity to create the tags (writing code to a single file for more than 30 seconds).
     */
    async fn create_activity_state_from_activities(
        &self,
        activities: Vec<Activity>,
        activity_period: ActivityPeriod,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        // iterate over the activities to create the start, end, context_switches, and activity_state_type
        log::trace!(
            "create_activity_state_from_activities: {}",
            activities.len()
        );

        if activities.is_empty() {
            log::trace!("  create_activity_state_from_activities: empty");
            self.create_idle_activity_state(activity_period).await
        } else {
            log::trace!("  create_activity_state_from_activities: not empty");
            // First lock: Get the context switches
            let context_switches = {
                let app_switch = APP_SWITCH_STATE.lock();
                app_switch.app_switches.clone()
            }; // lock is released here
            log::trace!("  context_switches: {:?}", context_switches);
            let result = self
                .activity_state_repo
                .create_active_activity_state(context_switches, &activity_period)
                .await;
            log::trace!("  created activity state");

            let activity_state = self
                .activity_state_service
                .get_last_activity_state()
                .await
                .expect("Failed to get activity state");

            // Only create tags if we have a valid activity state ID
            if let Some(activity_state_id) = activity_state.id {
                self.app_service
                    .create_tags_from_activities(&activities, activity_state_id)
                    .await
                    .expect("Failed to create activity state tags");
            } else {
                log::error!("Cannot create tags: activity state has no ID");
            }
            {
                let mut app_switch = APP_SWITCH_STATE.lock();
                app_switch.reset_app_switches();
            } // lock is released here
            log::trace!("  reset_app_switches");
            result
        }
    }

    #[cfg(test)]
    async fn get_last_activity_state(&self) -> Result<ActivityState, sqlx::Error> {
        self.activity_state_repo.get_last_activity_state().await
    }

    #[cfg(test)]
    async fn get_activity_starting_states_between(
        &self,
        start_time: OffsetDateTime,
        end_time: OffsetDateTime,
    ) -> Result<Vec<ActivityState>, sqlx::Error> {
        self.activity_state_repo
            .get_activity_states_starting_between(start_time, end_time)
            .await
    }

    pub fn start_activity_state_loop(&self, activity_state_interval: Duration) {
        let activity_service_clone = self.clone();
        let activity_state_service_clone = self.activity_state_service.clone();
        tokio::spawn(async move {
            let mut wait_interval = tokio::time::interval(activity_state_interval);
            wait_interval.tick().await;
            loop {
                log::trace!("tick");
                wait_interval.tick().await;
                let activities = activity_service_clone
                    .get_activities_since_last_activity_state()
                    .await
                    .unwrap();
                log::trace!("retrieved latest activities");
                let activity_period = activity_state_service_clone
                    .get_just_completed_activity_state(activity_state_interval)
                    .await;
                log::trace!("retrieved next activity state times");
                activity_service_clone
                    .create_activity_state_from_activities(activities, activity_period)
                    .await
                    .expect("Failed to create activity state");
                log::trace!("activity_state_created");
            }
        });
    }
}

pub async fn start_activities_monitoring(db_path: String) -> ActivityService {
    let db_manager = db_manager::DbManager::new(&db_path).await.unwrap();

    ActivityService::new(db_manager.pool)
}

#[cfg(test)]
mod tests {

    use os_monitor::{BlockedApp, Platform, WindowEvent};
    use time::OffsetDateTime;
    use uuid::Uuid;

    use super::*;
    use crate::db::{
        db_manager,
        models::{ActivityStateType, ActivityType},
    };

    #[tokio::test]
    async fn test_activity_service() {
        let pool = db_manager::create_test_db().await;
        let activity_service = ActivityService::new(pool.clone());
        let activity = Activity::__create_test_window(None, None);

        activity_service.save_activity(&activity).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_activity() {
        let pool = db_manager::create_test_db().await;
        let activity_service = ActivityService::new(pool.clone());
        let activity = Activity::__create_test_window(None, None);
        activity_service.save_activity(&activity).await.unwrap();

        let activity = activity_service.get_activity(1).await.unwrap();
        assert_eq!(
            activity.app_window_title,
            Some("main.rs - app-codeclimbers".to_string())
        );
    }

    #[tokio::test]
    async fn test_on_window_event_existing_app() {
        let pool = db_manager::create_test_db().await;
        let activity_service = ActivityService::new(pool);
        let event = WindowEvent {
            app_name: "Cursor".to_string(),
            window_title: "main.rs - app-codeclimbers".to_string(),
            url: None,
            bundle_id: Some("com.ebb.app".to_string()),
            platform: Platform::Mac,
        };
        activity_service.handle_window_activity(event).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let activity = activity_service.get_activity(1).await.unwrap();
        let app = activity_service
            .app_service
            .get_app_by_external_id("com.ebb.app")
            .await
            .unwrap();
        assert_eq!(activity.app_id, Some(app.id.unwrap()));
    }

    #[tokio::test]
    async fn test_on_window_event_new_app() {
        let pool = db_manager::create_test_db().await;
        let activity_service = ActivityService::new(pool);
        let event = WindowEvent {
            app_name: "New App".to_string(),
            window_title: "main.rs - app-codeclimbers".to_string(),
            url: None,
            bundle_id: Some("com.new.new".to_string()),
            platform: Platform::Mac,
        };
        activity_service.handle_window_activity(event).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let activity = activity_service.get_activity(1).await.unwrap();
        let app = activity_service
            .app_service
            .get_app_by_external_id("com.new.new")
            .await
            .unwrap();
        let tag = activity_service
            .app_service
            .get_app_tag_by_app_id(&app.id.clone().unwrap().to_string())
            .await
            .unwrap();
        assert_eq!(activity.app_id, Some(app.id.unwrap()));
        assert_eq!(tag.name, "neutral");
    }

    #[tokio::test]
    async fn test_on_window_event_new_app_has_url() {
        let pool = db_manager::create_test_db().await;
        let activity_service = ActivityService::new(pool);
        let event = WindowEvent {
            app_name: "New App".to_string(),
            window_title: "main.rs - app-codeclimbers".to_string(),
            url: Some("https://mail.google.com".to_string()),
            bundle_id: None,
            platform: Platform::Mac,
        };
        activity_service.handle_window_activity(event).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let activity = activity_service.get_activity(1).await.unwrap();
        let app = activity_service
            .app_service
            .get_app_by_external_id("mail.google.com")
            .await
            .unwrap();
        assert_eq!(activity.app_id, Some(app.id.unwrap()));
        assert_eq!(app.app_external_id, "mail.google.com");
    }

    #[tokio::test]
    async fn test_on_keyboard_event() {
        let pool = db_manager::create_test_db().await;
        let activity_service = ActivityService::new(pool);
        let event = KeyboardEvent {};
        activity_service.handle_keyboard_activity(event).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let activity = activity_service.get_activity(1).await.unwrap();
        assert_eq!(activity.activity_type, ActivityType::Keyboard);
    }

    #[tokio::test]
    async fn test_create_activity_state_from_activities_inactive() {
        let pool = db_manager::create_test_db().await;
        let activity_service = ActivityService::new(pool);
        let activities = vec![];
        let result = activity_service
            .create_activity_state_from_activities(
                activities,
                ActivityPeriod {
                    start_time: OffsetDateTime::now_utc(),
                    end_time: OffsetDateTime::now_utc() + Duration::from_secs(120),
                },
            )
            .await;
        assert!(result.is_ok());
        let activity_state = activity_service.get_last_activity_state().await.unwrap();
        assert_eq!(activity_state.state, ActivityStateType::Inactive);
        assert_eq!(activity_state.app_switches, 0);
    }

    #[tokio::test]
    async fn test_create_activity_state_from_activities_active() {
        let pool = db_manager::create_test_db().await;
        let activity_service = ActivityService::new(pool);
        let activities = vec![Activity::__create_test_window(
            None,
            Some(Uuid::new_v4().to_string()),
        )];
        let result = activity_service
            .create_activity_state_from_activities(
                activities,
                ActivityPeriod {
                    start_time: OffsetDateTime::now_utc(),
                    end_time: OffsetDateTime::now_utc() + Duration::from_secs(120),
                },
            )
            .await;
        assert!(result.is_ok());
        let activity_state = activity_service.get_last_activity_state().await.unwrap();
        assert_eq!(activity_state.state, ActivityStateType::Active);
        assert_eq!(activity_state.app_switches, 0);
    }

    #[tokio::test]
    async fn test_get_activities_since_last_activity_state_edge_time_case() {
        let pool = db_manager::create_test_db().await;
        let activity_service = ActivityService::new(pool);
        let now = OffsetDateTime::now_utc();

        // we have an activity at time 2 seconds ago.
        let mut activity = Activity::__create_test_window(None, None);
        activity.timestamp = Some(now - Duration::from_secs(2));
        activity_service.save_activity(&activity).await.unwrap();

        // we have an activity at time 1 second ago.
        let mut activity = Activity::__create_test_window(None, None);
        activity.timestamp = Some(now - Duration::from_secs(1));
        activity_service.save_activity(&activity).await.unwrap();

        // we have an activity at time 0 seconds ago.
        let mut activity = Activity::__create_test_window(None, None);
        activity.timestamp = Some(now);
        activity_service.save_activity(&activity).await.unwrap();

        // we have an activity_state that ended 1 second ago.
        let mut activity_state = ActivityState::new();
        activity_state.end_time = Some(now - Duration::from_secs(1));
        activity_service
            .save_activity_state(&activity_state)
            .await
            .unwrap();

        // retrieve activities since the last activity state
        let activities = activity_service
            .get_activities_since_last_activity_state()
            .await
            .unwrap();
        // should equal to 1 as the first activity is at time 2 seconds ago and the second activity is at time 1 second ago.
        assert_eq!(activities.len(), 1);

        // assert that the activities are from the second time window
    }

    #[tokio::test]
    async fn test_activity_state_loop() {
        let pool = db_manager::create_test_db().await;
        let activity_service = ActivityService::new(pool);
        let start = OffsetDateTime::now_utc();

        activity_service.start_activity_state_loop(Duration::from_millis(100));

        // Wait for a few iterations
        tokio::time::sleep(Duration::from_millis(350)).await;

        // Verify the results
        let activity_states = activity_service
            .get_activity_starting_states_between(start, OffsetDateTime::now_utc())
            .await
            .unwrap();

        assert_eq!(activity_states.len(), 3);
    }

    #[tokio::test]
    async fn test_handle_app_blocked_activity() {
        let pool = db_manager::create_test_db().await;
        let activity_service = ActivityService::new(pool.clone());

        // Create and handle two events with blocked apps
        let event1 = BlockedAppEvent {
            blocked_apps: vec![
                BlockedApp {
                    app_name: "Blocked App 1".to_string(),
                    app_external_id: "com.blocked.app1".to_string(),
                    is_site: false,
                },
                BlockedApp {
                    app_name: "Blocked App 2".to_string(),
                    app_external_id: "com.blocked.app2".to_string(),
                    is_site: true,
                },
            ],
        };

        let event2 = BlockedAppEvent {
            blocked_apps: vec![BlockedApp {
                app_name: "Blocked App 3".to_string(),
                app_external_id: "com.blocked.app3".to_string(),
                is_site: false,
            }],
        };

        activity_service.handle_app_blocked_activity(event1).await;
        activity_service.handle_app_blocked_activity(event2).await;

        // Use the repo to verify the records were saved
        let saved_records = activity_service
            .blocked_activity_repo
            .get_all_blocked_activities()
            .await
            .unwrap();

        assert_eq!(saved_records.len(), 3); // Should have 3 records total
        assert_eq!(saved_records[0].external_app_id, "com.blocked.app1");
        assert_eq!(saved_records[1].external_app_id, "com.blocked.app2");
        assert_eq!(saved_records[2].external_app_id, "com.blocked.app3");

        // Verify timestamps are set
        for record in &saved_records {
            assert!(record.created_at.is_some());
            assert!(record.updated_at.is_some());
        }
    }
}
