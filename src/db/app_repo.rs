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
            r#"INSERT INTO app (id, name, app_id, platform, is_browser, is_default) 
            VALUES (?, ?, ?, ?, ?, ?)"#,
            app.id,
            app.name,
            app.app_id,
            app.platform as _,
            app.is_browser,
            app.is_default,
        )
        .execute(&mut *conn)
        .await
    }

    pub async fn get_app_by_name_or_url(&self, name: &str, url: &str) -> Result<App, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query_as!(
            App,
            r#"SELECT id, name, app_id, platform, is_browser, is_default, is_blocked, created_at, updated_at
            FROM app WHERE name = ? OR name = ?"#,
            name,
            url
        )
        .fetch_one(&mut *conn)
        .await
    }

    pub async fn get_apps_by_app_ids(
        &self,
        app_ids: &Vec<String>,
    ) -> Result<Vec<App>, sqlx::Error> {
        if app_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = self.pool.acquire().await?;

        // Create the parameterized query with the correct number of placeholders
        let placeholders = std::iter::repeat("?")
            .take(app_ids.len())
            .collect::<Vec<_>>()
            .join(",");

        let query = format!(
            r#"SELECT id, name, app_id, platform, is_browser, is_default, is_blocked, created_at, updated_at
            FROM app WHERE app_id IN ({})"#,
            placeholders
        );

        // Build the query with dynamic parameters
        let mut query = sqlx::query_as::<_, App>(&query);

        // Bind each parameter
        for app_id in app_ids {
            query = query.bind(app_id);
        }

        query.fetch_all(&mut *conn).await
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
