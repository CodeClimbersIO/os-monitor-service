use std::collections::HashSet;

use sqlx::SqlitePool;

use super::models::{App, Tag};

#[derive(Clone)]
pub struct TagRepo {
    pool: SqlitePool,
}

impl TagRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_tag_by_name(&self, name: &str) -> Result<Tag, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;

        sqlx::query_as!(Tag, "SELECT * FROM tag WHERE name = ?", name)
            .fetch_one(&mut *conn)
            .await
    }

    pub async fn get_default_tags_by_app(&self, apps: &Vec<App>) -> Result<Vec<Tag>, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        let app_ids = apps
            .iter()
            .filter_map(|app| app.id.clone())
            .collect::<Vec<String>>();
        let placeholders = std::iter::repeat("?")
            .take(app_ids.len())
            .collect::<Vec<_>>()
            .join(",");

        let query = format!(
            r#"
                SELECT tag.*
                FROM tag 
                JOIN app_tag ON tag.id = app_tag.tag_id 
                WHERE app_tag.app_id IN ({}) AND tag.tag_type = 'default'"#,
            placeholders
        );

        let mut query = sqlx::query_as::<_, Tag>(&query);

        for app_id in app_ids {
            query = query.bind(app_id);
        }

        query.fetch_all(&mut *conn).await
    }

    pub async fn create_tag(&self, name: &str, tag_type: &str) -> Result<(), sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query!(
            "INSERT INTO tag (name, tag_type) VALUES (?, ?)",
            name,
            tag_type
        )
        .execute(&mut *conn)
        .await?;
        Ok(())
    }

    pub async fn create_activity_state_tags(
        &self,
        activity_state_id: i64,
        tags: &Vec<Tag>,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        let unique_tags = tags
            .iter()
            .filter_map(|tag| tag.id.clone())
            .collect::<HashSet<String>>();

        let placeholders = std::iter::repeat("(?, ?)")
            .take(unique_tags.len())
            .collect::<Vec<_>>()
            .join(",");

        let query = format!(
            r#"
            INSERT INTO activity_state_tag (activity_state_id, tag_id)
            VALUES {}"#,
            placeholders
        );

        let mut query = sqlx::query(&query);
        for tag in unique_tags {
            query = query.bind(activity_state_id).bind(tag);
        }

        query.execute(&mut *conn).await
    }

    pub async fn create_app_tag(
        &self,
        app_id: String,
        tag_id: String,
        weight: f32,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        let id = uuid::Uuid::new_v4().to_string();

        sqlx::query!(
            "INSERT INTO app_tag (id, app_id, tag_id, weight) VALUES (?, ?, ?, ?)",
            id,
            app_id,
            tag_id,
            weight
        )
        .execute(&mut *conn)
        .await
    }

    #[cfg(test)]
    pub async fn get_tags_for_activity_state(
        &self,
        activity_state_id: i64,
    ) -> Result<Vec<Tag>, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query_as!(
            Tag,
            "SELECT tag.* FROM tag
            JOIN activity_state_tag ON tag.id = activity_state_tag.tag_id
            WHERE activity_state_tag.activity_state_id = ?",
            activity_state_id
        )
        .fetch_all(&mut *conn)
        .await
    }
}
