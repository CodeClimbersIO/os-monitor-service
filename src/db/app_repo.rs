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

    pub async fn get_apps_by_name(&self, names: &Vec<String>) -> Result<Vec<App>, sqlx::Error> {
        if names.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = self.pool.acquire().await?;

        // Create the parameterized query with the correct number of placeholders
        let placeholders = std::iter::repeat("?")
            .take(names.len())
            .collect::<Vec<_>>()
            .join(",");

        let query = format!(
            r#"SELECT id, name, platform, is_browser, created_at, updated_at
            FROM app WHERE name IN ({})"#,
            placeholders
        );

        // Build the query with dynamic parameters
        let mut query = sqlx::query_as::<_, App>(&query);

        // Bind each parameter
        for name in names {
            query = query.bind(name);
        }

        query.fetch_all(&mut *conn).await
    }

    pub async fn begin_transaction(
        &self,
    ) -> Result<sqlx::Transaction<'_, sqlx::Sqlite>, sqlx::Error> {
        self.pool.begin().await
    }

    pub async fn upsert_app(
        &self,
        name: &str,
        platform: &str,
        is_browser: bool,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<i64>, sqlx::Error> {
        // Upsert app
        sqlx::query!(
            r#"
            INSERT INTO app (name, platform, is_browser)
            VALUES (?, ?, ?)
            ON CONFLICT (name, platform) DO UPDATE SET
                is_browser = excluded.is_browser
                WHERE app.name = excluded.name
                AND app.platform = excluded.platform
            "#,
            name,
            platform,
            is_browser
        )
        .execute(&mut **tx)
        .await?;

        // Get app_id
        let app = sqlx::query!(
            "SELECT id FROM app WHERE name = ? AND platform = ?",
            name,
            platform
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(app.id)
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
