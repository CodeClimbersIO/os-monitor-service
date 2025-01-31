use super::models::App;
#[derive(Clone)]
pub struct AppRepo {
    pool: sqlx::SqlitePool,
}

impl AppRepo {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        AppRepo { pool }
    }

    pub async fn save_app(
        &self,
        app: &App,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query!(
            r#"INSERT INTO app (name, platform, is_browser) 
            VALUES (?, ?, ?)"#,
            app.name,
            app.platform as _,
            app.is_browser,
        )
        .execute(&mut *conn)
        .await
    }

    pub async fn get_app_by_name_or_url(&self, name: &str, url: &str) -> Result<App, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query_as!(
            App,
            r#"SELECT id, name, platform, is_browser, created_at, updated_at
            FROM app WHERE name = ? OR name = ?"#,
            name,
            url
        )
        .fetch_one(&mut *conn)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::db_manager;
    #[tokio::test]
    async fn test_app_repo() {
        let pool = db_manager::create_test_db().await;
        let app_repo = AppRepo::new(pool);
        let app = App::__create_test_app();
        app_repo.save_app(&app).await.unwrap();
    }
}
