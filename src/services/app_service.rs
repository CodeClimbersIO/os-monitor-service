use os_monitor::WindowEvent;

use crate::db::{
    activity_repo::ActivityRepo,
    app_repo::AppRepo,
    models::{Activity, ActivityType, App},
    tag_repo::TagRepo,
};

#[cfg(test)]
use crate::db::models::Tag;

#[derive(Clone)]
pub struct AppService {
    app_repo: AppRepo,
    tag_repo: TagRepo,
    activity_repo: ActivityRepo,
}

impl AppService {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        AppService {
            app_repo: AppRepo::new(pool.clone()),
            tag_repo: TagRepo::new(pool.clone()),
            activity_repo: ActivityRepo::new(pool.clone()),
        }
    }

    pub async fn create_idle_tag(
        &self,
        activity_state_id: i64,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let idle_tag = self
            .tag_repo
            .get_tag_by_name("idle")
            .await
            .expect("Failed to get idle tag");
        self.tag_repo
            .create_activity_state_tags(activity_state_id, &vec![idle_tag])
            .await
    }

    pub async fn create_tags_from_activities(
        &self,
        activities: &Vec<Activity>,
        activity_state_id: i64,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        log::trace!("    Creating Tags From Activities");
        // get apps from activities
        let mut app_ids = activities
            .iter()
            .filter_map(|a| a.app_id.clone())
            .collect::<Vec<String>>();
        // if there are no names, use the latest window type activity
        if app_ids.is_empty() {
            let latest_activity = self
                .activity_repo
                .get_last_activity_by_type(ActivityType::Window)
                .await
                .expect("Failed to get last activity");
            log::trace!(
                "      no window changes, likely were in last activity: {:?}",
                latest_activity
            );
            let app_id = latest_activity.app_id.clone();
            if let Some(app_id) = app_id {
                app_ids.push(app_id);
            }
        }

        let app_tags = self
            .app_repo
            .get_app_tag_by_app_ids(&app_ids)
            .await
            .expect("Failed to get apps");

        log::trace!("    apps: {:?}", app_tags);

        // for each tag, create a tag_activity_state_mapping
        return self
            .tag_repo
            .create_activity_state_tags_with_app_tags(activity_state_id, &app_tags)
            .await;
    }

    /**
     * When we get a new window event, we have some behavior to handle apps.
     * Apps are identified by their external id (either the url or the bundle id from the event)
     * If the app exists in the database (by external id), we return the app.id
     * If the app does not exist, we create a new app and a default tag for it and return the new app.id
     */
    pub async fn handle_window_event(&self, event: &WindowEvent) -> Result<String, sqlx::Error> {
        let raw_app = App::new(&event);
        let app = self.get_app_by_external_id(&raw_app.app_external_id).await;
        if let Ok(app) = app {
            return Ok(app.id.unwrap());
        } else {
            log::trace!("app not found, creating new app");
            match self.save_app(&raw_app).await {
                Ok(_) => {
                    if let Err(err) = self.create_default_app_tag(raw_app.id.clone()).await {
                        log::error!("Failed to create default tag for app: {}", err);
                    }
                    Ok(raw_app.id.clone().unwrap())
                }
                Err(err) => Err(err),
            }
        }
    }

    async fn create_default_app_tag(
        &self,
        app_id: Option<String>,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let neutral_tag = self
            .tag_repo
            .get_tag_by_name("neutral")
            .await
            .expect("Failed to get neutral tag");
        log::trace!("neutral_tag: {:?}", neutral_tag);
        if let (Some(app_id), Some(tag_id)) = (app_id, neutral_tag.id.clone()) {
            log::trace!("app_id: {:?}", app_id);
            log::trace!("tag_id: {:?}", tag_id);
            self.tag_repo.create_app_tag(app_id, tag_id, 1.0).await
        } else {
            Err(sqlx::Error::RowNotFound)
        }
    }

    pub async fn get_app_by_external_id(&self, external_app_id: &str) -> Result<App, sqlx::Error> {
        self.app_repo.get_app_by_external_id(external_app_id).await
    }

    pub async fn save_app(
        &self,
        app: &App,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        self.app_repo.save_app(app).await
    }

    #[cfg(test)]
    pub async fn get_app_tag_by_app_id(&self, app_id: &str) -> Result<Tag, sqlx::Error> {
        self.tag_repo.get_app_tag_by_app_id(app_id).await
    }

    #[cfg(test)]
    pub async fn get_tags_for_activity_state(
        &self,
        activity_state_id: i64,
    ) -> Result<Vec<Tag>, sqlx::Error> {
        self.tag_repo
            .get_tags_for_activity_state(activity_state_id)
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        db::{
            activity_state_repo::ActivityStateRepo,
            db_manager,
            models::{ActivityState, ActivityStateType},
        },
        services::activities_service::ActivityService,
    };
    use os_monitor::Platform as OsPlatform;
    use time::OffsetDateTime;

    use super::*;

    #[tokio::test]
    async fn test_create_tags_from_activities_only_one_activity() {
        let pool = db_manager::create_test_db().await;
        let app_service = AppService::new(pool.clone());

        let activity_state_repo = ActivityStateRepo::new(pool.clone());
        let activity_state = ActivityState {
            id: None,
            state: ActivityStateType::Inactive,
            app_switches: 0,
            start_time: Some(OffsetDateTime::now_utc()),
            end_time: Some(OffsetDateTime::now_utc()),
            created_at: Some(OffsetDateTime::now_utc()),
        };
        activity_state_repo
            .save_activity_state(&activity_state)
            .await
            .expect("Failed to save activity state");
        let activity_service = ActivityService::new(pool.clone());
        let event = WindowEvent {
            app_name: "Cursor".to_string(),
            window_title: "main.rs - app-codeclimbers".to_string(),
            url: None,
            platform: OsPlatform::Mac,
            bundle_id: Some("com.todesktop.230313mzl4w4u92".to_string()),
        };
        activity_service.handle_window_activity(event).await;

        let activities = activity_service
            .get_activities_since_last_activity_state()
            .await
            .expect("Failed to get activities");
        let activity_state = activity_state_repo
            .get_last_activity_state()
            .await
            .expect("Failed to get activity state");
        app_service
            .create_tags_from_activities(&activities, activity_state.id.unwrap())
            .await
            .expect("Failed to create tags");

        let tags = app_service
            .get_tags_for_activity_state(activity_state.id.unwrap())
            .await
            .expect("Failed to get tags");

        assert_eq!(tags.len(), 2); // just cursor should end up with the tags for creating and coding
    }

    #[tokio::test]
    async fn test_create_tags_from_activities_only_with_app_ids() {
        let pool = db_manager::create_test_db().await;
        let app_service = AppService::new(pool.clone());

        let activity_state_repo = ActivityStateRepo::new(pool.clone());
        let activity_state = ActivityState {
            id: None,
            state: ActivityStateType::Inactive,
            app_switches: 0,
            start_time: Some(OffsetDateTime::now_utc()),
            end_time: Some(OffsetDateTime::now_utc()),
            created_at: Some(OffsetDateTime::now_utc()),
        };
        activity_state_repo
            .save_activity_state(&activity_state)
            .await
            .expect("Failed to save activity state");
        let activity_service = ActivityService::new(pool.clone());
        let event = WindowEvent {
            app_name: "Cursor".to_string(),
            window_title: "main.rs - app-codeclimbers".to_string(),
            url: None,
            platform: OsPlatform::Mac,
            bundle_id: Some("com.todesktop.230313mzl4w4u92".to_string()),
        };
        activity_service.handle_window_activity(event).await;
        let event = WindowEvent {
            app_name: "Google Chrome".to_string(),
            window_title: "Google".to_string(),
            url: Some("google.com".to_string()),
            platform: OsPlatform::Mac,
            bundle_id: None,
        };
        activity_service.handle_window_activity(event).await;
        let event = WindowEvent {
            app_name: "Google Chrome".to_string(),
            window_title: "X - Twitter".to_string(),
            url: Some("x.com".to_string()),
            platform: OsPlatform::Mac,
            bundle_id: None,
        };
        activity_service.handle_window_activity(event).await;
        let event = WindowEvent {
            app_name: "Ebb".to_string(),
            window_title: "main".to_string(),
            url: None,
            platform: OsPlatform::Mac,
            bundle_id: Some("com.ebb.app".to_string()),
        };
        activity_service.handle_window_activity(event).await;
        let event = WindowEvent {
            app_name: "Google Chrome".to_string(),
            window_title: "Instagram".to_string(),
            url: Some("instagram.com".to_string()),
            platform: OsPlatform::Mac,
            bundle_id: None,
        };
        activity_service.handle_window_activity(event).await;

        let activities = activity_service
            .get_activities_since_last_activity_state()
            .await
            .expect("Failed to get activities");
        let activity_state = activity_state_repo
            .get_last_activity_state()
            .await
            .expect("Failed to get activity state");
        app_service
            .create_tags_from_activities(&activities, activity_state.id.unwrap())
            .await
            .expect("Failed to create tags");

        let tags = app_service
            .get_tags_for_activity_state(activity_state.id.unwrap())
            .await
            .expect("Failed to get tags");

        assert_eq!(tags.len(), 10);
    }
}
