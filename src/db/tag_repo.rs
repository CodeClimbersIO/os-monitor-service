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

    pub async fn ensure_default_tag(
        &self,
        tag_name: &str,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO tag (name, tag_type)
            VALUES (?, 'default')
            ON CONFLICT (name, tag_type) DO UPDATE SET
                tag_type = 'default'
                WHERE tag.name = ?
            "#,
            tag_name,
            tag_name
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn get_tag_by_name(
        &self,
        name: &str,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<i64>, sqlx::Error> {
        let record = sqlx::query!("SELECT id FROM tag WHERE name = ?", name)
            .fetch_one(&mut **tx)
            .await?;

        Ok(record.id)
    }

    pub async fn get_tags_by_app(&self, apps: &Vec<App>) -> Result<Vec<Tag>, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        let app_ids = apps.iter().filter_map(|app| app.id).collect::<Vec<i64>>();
        let placeholders = std::iter::repeat("?")
            .take(app_ids.len())
            .collect::<Vec<_>>()
            .join(",");

        let query = format!(
            r#"
                SELECT *
                FROM tag 
                JOIN app_tag ON tag.id = app_tag.tag_id 
                WHERE app_tag.app_id IN ({})"#,
            placeholders
        );

        let mut query = sqlx::query_as::<_, Tag>(&query);

        for app_id in app_ids {
            query = query.bind(app_id);
        }

        query.fetch_all(&mut *conn).await
    }

    pub async fn create_app_tag_relation(
        &self,
        app_id: i64,
        tag_id: i64,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO app_tag (app_id, tag_id, weight)
            VALUES (?, ?, 1.0)
            ON CONFLICT (app_id, tag_id) DO UPDATE SET
                weight = 1.0
                WHERE app_tag.app_id = excluded.app_id
                AND app_tag.tag_id = excluded.tag_id
            "#,
            app_id,
            tag_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn create_activity_state_tags(
        &self,
        activity_state_id: i64,
        tags: &Vec<Tag>,
    ) -> Result<(), sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        let unique_tags = tags
            .iter()
            .filter_map(|tag| tag.id)
            .collect::<HashSet<i64>>();

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

        query.execute(&mut *conn).await?;
        Ok(())
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
