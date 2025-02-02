use colored::*;
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
        println!("{}", "    Creating Tags From Activities".blue());
        // get apps from activities
        let mut names = activities
            .iter()
            .filter_map(|a| a.app_name.clone())
            .collect::<Vec<String>>();
        let urls = activities
            .iter()
            .filter_map(|a| a.url.clone())
            .collect::<Vec<String>>();

        let apps = urls
            .iter()
            .map(|url| App::get_domain_from_url(url))
            .collect::<Vec<String>>();

        names.extend(apps);

        // if there are no names, use the latest window type activity
        if names.is_empty() {
            let latest_activity = self
                .activity_repo
                .get_last_activity_by_type(ActivityType::Window)
                .await
                .expect("Failed to get last activity");
            println!(
                "      {}: {:?}",
                "no window changes, likely were in last activity".blue(),
                latest_activity
            );
            let name = latest_activity.app_name.clone();
            let url = latest_activity.url.clone();
            if let Some(name) = name {
                names.push(name);
            }
            if let Some(url) = url {
                names.push(url);
            }
        }

        let apps = self
            .app_repo
            .get_apps_by_name(&names)
            .await
            .expect("Failed to get apps");

        println!("    {}: {:?}", "apps".blue(), apps);
        // get all related tags to those apps
        let mut tags = self
            .tag_repo
            .get_tags_by_app(&apps)
            .await
            .expect("Failed to get tags");
        println!("    {}: {:?}", "tags".blue(), tags);
        // for each tag, create a tag_activity_state_mapping
        if tags.is_empty() {
            println!("      {}: {:?}", "no tags found for apps".blue(), apps);
            tags = vec![self
                .tag_repo
                .get_tag_by_name("consuming")
                .await
                .expect("Failed to get consuming tag")];
        }
        return self
            .tag_repo
            .create_activity_state_tags(activity_state_id, &tags)
            .await;
    }

    pub async fn handle_window_event(&self, event: &WindowEvent) {
        let app = App::new(&event);
        match self.save_app(&app).await {
            Ok(_) => {
                let last_created_app = self
                    .get_last_created_app()
                    .await
                    .expect("Failed to get last created app");
                if let Err(err) = self.create_default_app_tag(&last_created_app).await {
                    eprintln!("Failed to create default tag for app: {}", err);
                }
            }
            Err(err) => {
                match err {
                    sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("2067") => {
                        // Silently ignore duplicate entries
                        return;
                    }
                    other_err => {
                        // Log or handle other database errors
                        eprintln!("Failed to save app: {}", other_err);
                    }
                }
            }
        }
    }

    // Add this new helper method
    async fn create_default_app_tag(
        &self,
        app: &App,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let consuming_tag = self
            .tag_repo
            .get_tag_by_name("consuming")
            .await
            .expect("Failed to get consuming tag");
        println!("consuming_tag: {:?}", consuming_tag);
        println!("app: {:?}", app);
        if let (Some(app_id), Some(tag_id)) = (app.id, consuming_tag.id) {
            println!("app_id: {:?}", app_id);
            println!("tag_id: {:?}", tag_id);
            self.tag_repo.create_app_tag(app_id, tag_id, 1.0).await
        } else {
            Err(sqlx::Error::RowNotFound)
        }
    }

    pub async fn save_app(
        &self,
        app: &App,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        self.app_repo.save_app(app).await
    }

    pub async fn get_last_created_app(&self) -> Result<App, sqlx::Error> {
        self.app_repo.get_last_created_app().await
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
    async fn test_create_tags_from_activities_only_with_app_names() {
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
        };
        activity_service.handle_window_activity(event).await;
        let event = WindowEvent {
            app_name: "Google Chrome".to_string(),
            window_title: "main.rs - app-codeclimbers".to_string(),
            url: Some("https://www.google.com/".to_string()),
            platform: OsPlatform::Mac,
        };
        activity_service.handle_window_activity(event).await;
        let event = WindowEvent {
            app_name: "Google Chrome".to_string(),
            window_title: "main.rs - app-codeclimbers".to_string(),
            url: Some("https://www.x.com/".to_string()),
            platform: OsPlatform::Mac,
        };
        activity_service.handle_window_activity(event).await;
        let event = WindowEvent {
            app_name: "Ebb".to_string(),
            window_title: "main".to_string(),
            url: None,
            platform: OsPlatform::Mac,
        };
        activity_service.handle_window_activity(event).await;
        let event = WindowEvent {
            app_name: "Google Chrome".to_string(),
            window_title: "main".to_string(),
            url: Some("https://www.instagram.com/your_page".to_string()),
            platform: OsPlatform::Mac,
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
        println!("tags: {:?}", tags);
        assert_eq!(tags.len(), 2);
    }
}
