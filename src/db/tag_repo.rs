use std::collections::HashSet;

use sqlx::SqlitePool;

use super::models::{AppTag, Tag};

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

    pub async fn get_default_tags_by_app_tags(
        &self,
        app_tags: &Vec<AppTag>,
    ) -> Result<Vec<Tag>, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        let app_ids = app_tags
            .iter()
            .filter_map(|app_tag| Some(app_tag.app_id.clone()))
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

    pub async fn create_activity_state_tags_with_app_tags(
        &self,
        activity_state_id: i64,
        app_tags: &Vec<AppTag>,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;

        // Create unique pairs of (app_tag.id, app_tag.tag_id)
        let unique_tags = app_tags
            .iter()
            .map(|app_tag| (app_tag.id.clone(), app_tag.tag_id.clone()))
            .collect::<HashSet<(Option<String>, String)>>();

        let placeholders = std::iter::repeat("(?, ?, ?)")
            .take(unique_tags.len())
            .collect::<Vec<_>>()
            .join(",");

        if unique_tags.is_empty() {
            println!("DEBUG: No app_tags to insert, returning early");
            // Return a successful result but with 0 rows affected by executing a no-op query
            return sqlx::query("SELECT 1 WHERE 0").execute(&mut *conn).await;
        }

        let query = format!(
            r#"
            INSERT INTO activity_state_tag (activity_state_id, app_tag_id, tag_id)
            VALUES {}"#,
            placeholders
        );

        let mut query = sqlx::query(&query);
        for (app_tag_id, tag_id) in unique_tags {
            query = query.bind(activity_state_id).bind(app_tag_id).bind(tag_id);
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
    pub async fn get_app_tag_by_app_id(&self, app_id: &str) -> Result<Tag, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query_as!(
            Tag,
            "SELECT tag.* FROM tag JOIN app_tag ON tag.id = app_tag.tag_id WHERE app_tag.app_id = ?",
            app_id
        )
        .fetch_one(&mut *conn)
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

#[cfg(test)]
mod tests {
    // use crate::db::{db_manager, models::AppTag, tag_repo::TagRepo};

    // #[tokio::test]
    // async fn test_create_activity_state_tags() {
    //     let pool = db_manager::create_test_db().await;
    //     let tag_repo = TagRepo::new(pool);

    //     let app_tags = vec![AppTag::__create_test_app_tag(
    //         "".to_string(),
    //         "".to_string(),
    //     )];

    //     let activity_state_id = 1;

    //     let res = tag_repo
    //         .create_activity_state_tags_with_app_tags(activity_state_id, &app_tags)
    //         .await;

    //     // let activity = activity_service.get_activity(1).await.unwrap();
    //     assert_eq!(true, true);
    // }
}
