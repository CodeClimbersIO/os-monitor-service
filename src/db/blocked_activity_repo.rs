use super::models::BlockedActivity;

#[derive(Clone)]
pub struct BlockedActivityRepo {
    pool: sqlx::SqlitePool,
}

impl BlockedActivityRepo {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        BlockedActivityRepo { pool }
    }

    pub async fn save_blocked_activity(
        &self,
        blocked_activity: &BlockedActivity,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let mut conn = self.pool.acquire().await?;
        sqlx::query!(
            r#"INSERT INTO blocked_activity (id, external_app_id, created_at, updated_at) 
            VALUES (?, ?, ?, ?)"#,
            blocked_activity.id,
            blocked_activity.external_app_id,
            blocked_activity.created_at,
            blocked_activity.updated_at,
        )
        .execute(&mut *conn)
        .await
    }

    pub async fn get_all_blocked_activities(&self) -> Result<Vec<BlockedActivity>, sqlx::Error> {
        sqlx::query_as!(
            BlockedActivity,
            r#"SELECT id, external_app_id, created_at, updated_at 
            FROM blocked_activity 
            ORDER BY created_at"#
        )
        .fetch_all(&self.pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::db_manager;
    use time::OffsetDateTime;

    #[tokio::test]
    async fn test_blocked_activity_repo() {
        let pool = db_manager::create_test_db().await;
        let blocked_activity_repo = BlockedActivityRepo::new(pool);

        // Create and save two blocked activities
        let mut blocked_activity1 = BlockedActivity::__create_test_blocked_activity();
        blocked_activity1.external_app_id = "com.blocked.app1".to_string();
        blocked_activity1.created_at = Some(OffsetDateTime::now_utc());
        blocked_activity1.updated_at = Some(OffsetDateTime::now_utc());

        let mut blocked_activity2 = BlockedActivity::__create_test_blocked_activity();
        blocked_activity2.external_app_id = "com.blocked.app2".to_string();
        blocked_activity2.created_at = Some(OffsetDateTime::now_utc());
        blocked_activity2.updated_at = Some(OffsetDateTime::now_utc());

        blocked_activity_repo
            .save_blocked_activity(&blocked_activity1)
            .await
            .unwrap();
        blocked_activity_repo
            .save_blocked_activity(&blocked_activity2)
            .await
            .unwrap();

        // Fetch and verify the saved records
        let saved_records = blocked_activity_repo
            .get_all_blocked_activities()
            .await
            .unwrap();
        assert_eq!(saved_records.len(), 2);
        assert_eq!(saved_records[0].external_app_id, "com.blocked.app1");
        assert_eq!(saved_records[1].external_app_id, "com.blocked.app2");
    }
}
