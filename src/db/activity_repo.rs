// activity repo is responsible for all the database operations related to activities. Makes use of the db manager to get the pool and execute queries.

use super::models::{Activity, ActivityType};
#[derive(Clone)]
pub struct ActivityRepo {
    pool: sqlx::SqlitePool,
}

impl ActivityRepo {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        ActivityRepo { pool }
    }

    pub async fn get_last_activity(&self) -> Result<Activity, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query_as!(
            Activity,
            r#"SELECT id, created_at, timestamp, activity_type as "activity_type: _", app_id, app_window_title, platform as "platform: _" FROM activity ORDER BY id DESC LIMIT 1"#
        )
        .fetch_one(&mut *conn)
        .await
    }

    pub async fn save_activity(
        &self,
        activity: &Activity,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let app_id = if activity.app_id.is_none() {
            // if no app_id, try to get the last known app_id, defaulting to None if no previous activity exists
            match self.get_last_activity().await {
                Ok(last_activity) => last_activity.app_id,
                Err(_) => None,
            }
        } else {
            activity.app_id.clone()
        };

        let mut conn = self.pool.acquire().await?;
        sqlx::query!(
            r#"INSERT INTO activity (activity_type, app_id, app_window_title, timestamp, platform) 
            VALUES (?, ?, ?, ?, ?)"#,
            activity.activity_type as _,
            app_id,
            activity.app_window_title,
            activity.timestamp,
            activity.platform as _,
        )
        .execute(&mut *conn)
        .await
    }

    pub async fn get_activity(&self, id: i32) -> Result<Activity, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query_as!(
            Activity,
            r#"SELECT id, created_at, timestamp, activity_type as "activity_type: _", 
            app_id, app_window_title, platform as "platform: _" 
            FROM activity WHERE id = ?"#,
            id
        )
        .fetch_one(&mut *conn)
        .await
    }

    pub async fn get_last_activity_by_type(
        &self,
        activity_type: ActivityType,
    ) -> Result<Activity, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query_as!(
            Activity,
            r#"SELECT id, created_at, timestamp, 
                   activity_type as "activity_type: _",
                   app_id, app_window_title, platform as "platform: _" 
                   FROM activity WHERE activity_type = ? ORDER BY timestamp DESC LIMIT 1"#,
            activity_type as _
        )
        .fetch_one(&mut *conn)
        .await
    }

    // get the activities since the last activity state. If none, return an empty vector.
    pub(crate) async fn get_activities_since_last_activity_state(
        &self,
    ) -> Result<Vec<Activity>, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;

        sqlx::query_as!(
            Activity,
            r#"
            SELECT id, created_at, timestamp, 
                activity_type as "activity_type: _",
                app_id, app_window_title, 
                platform as "platform: _"
            FROM activity a
                WHERE a.timestamp > (
                    SELECT end_time 
                    FROM activity_state 
                    ORDER BY id DESC LIMIT 1
                )
            ORDER BY a.timestamp ASC
            "#
        )
        .fetch_all(&mut *conn)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::db_manager;
    #[tokio::test]
    async fn test_activity_repo() {
        let pool = db_manager::create_test_db().await;
        let activity_repo = ActivityRepo::new(pool);
        let activity = Activity::__create_test_window(None, None);
        activity_repo.save_activity(&activity).await.unwrap();
    }
}
