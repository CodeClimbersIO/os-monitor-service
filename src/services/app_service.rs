use os_monitor::WindowEvent;

use crate::db::{app_repo::AppRepo, models::App};

#[derive(Clone)]
pub struct AppService {
    app_repo: AppRepo,
}

impl AppService {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        AppService {
            app_repo: AppRepo::new(pool.clone()),
        }
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
}
