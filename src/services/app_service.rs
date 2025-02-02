use os_monitor::WindowEvent;

use crate::db::{
    app_repo::AppRepo,
    models::{Activity, App},
    tag_repo::TagRepo,
};

#[cfg(test)]
use crate::db::models::Tag;

#[derive(Clone)]
pub struct AppService {
    app_repo: AppRepo,
    tag_repo: TagRepo,
}

impl AppService {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        AppService {
            app_repo: AppRepo::new(pool.clone()),
            tag_repo: TagRepo::new(pool),
        }
    }

    pub async fn create_tags_from_activities(
        &self,
        activities: &Vec<Activity>,
        activity_state_id: i64,
    ) -> Result<(), sqlx::Error> {
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
        println!("extended names: {:?}", names);
        let apps = self
            .app_repo
            .get_apps_by_name(&names)
            .await
            .expect("Failed to get apps");
        println!("apps: {:?}", apps);
        // get all related tags to those apps
        let tags = self
            .tag_repo
            .get_tags_by_app(&apps)
            .await
            .expect("Failed to get tags");
        println!("tags: {:?}", tags);
        // for each tag, create a tag_activity_state_mapping
        return self
            .tag_repo
            .create_activity_state_tags(activity_state_id, &tags)
            .await;
    }

    pub async fn handle_window_event(&self, event: &WindowEvent) {
        let app = App::new(&event);
        if let Err(err) = self.save_app(&app).await {
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

    pub async fn save_app(
        &self,
        app: &App,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        self.app_repo.save_app(app).await
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

impl AppService {
    pub async fn import_apps_from_csv(&self, csv_path: &str) -> Result<(), sqlx::Error> {
        // Begin transaction
        let mut tx = self.app_repo.begin_transaction().await?;

        // Ensure default tags exist
        for tag_name in ["creating", "consuming"] {
            self.tag_repo.ensure_default_tag(tag_name, &mut tx).await?;
        }

        // Read CSV file
        let file = std::fs::File::open(csv_path).map_err(|e| {
            sqlx::Error::Configuration(format!("Failed to open CSV file: {}", e).into())
        })?;
        let mut rdr = csv::Reader::from_reader(file);

        // Process each record
        for result in rdr.records() {
            let record = result.map_err(|e| {
                sqlx::Error::Configuration(format!("Failed to read CSV record: {}", e).into())
            })?;

            let name = record
                .get(0)
                .ok_or_else(|| sqlx::Error::Configuration("Missing name column".into()))?
                .to_string();
            let platform = record
                .get(1)
                .ok_or_else(|| sqlx::Error::Configuration("Missing platform column".into()))?
                .to_string();
            let is_browser = record
                .get(2)
                .ok_or_else(|| sqlx::Error::Configuration("Missing is_browser column".into()))?
                == "1";
            let tag = record
                .get(3)
                .ok_or_else(|| sqlx::Error::Configuration("Missing tag column".into()))?
                .to_string();

            println!(
                "name: {}, platform: {}, is_browser: {}, tag: {}",
                name, platform, is_browser, tag
            );
            // Upsert app and get its ID
            let app_id = self
                .app_repo
                .upsert_app(&name, &platform, is_browser, &mut tx)
                .await?;

            // Get tag ID
            let tag_id = self.tag_repo.get_tag_by_name(&tag, &mut tx).await?;

            if let (Some(app_id), Some(tag_id)) = (app_id, tag_id) {
                // Create app-tag relation
                self.tag_repo
                    .create_app_tag_relation(app_id, tag_id, &mut tx)
                    .await?;
            }
        }

        // Commit transaction
        tx.commit().await?;
        Ok(())
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
        app_service
            .import_apps_from_csv("src/services/apps.csv")
            .await
            .expect("Failed to import apps");
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
